//! Wasm compilation orchestration.
//!
//! It works roughly like this:
//!
//! * We walk over the Wasm module/component and make a list of all the things
//!   we need to compile. This is a `CompileInputs`.
//!
//! * The `CompileInputs::compile` method compiles each of these in parallel,
//!   producing a `UnlinkedCompileOutputs`. This is an unlinked set of compiled
//!   functions, bucketed by type of function.
//!
//! * The `UnlinkedCompileOutputs::pre_link` method re-arranges the compiled
//!   functions into a flat list. This is the order we will place them within
//!   the ELF file, so we must also keep track of all the functions' indices
//!   within this list, because we will need them for resolving
//!   relocations. These indices are kept track of in the resulting
//!   `FunctionIndices`.
//!
//! * The `FunctionIndices::link_and_append_code` method appends the functions
//!   to the given ELF file and resolves relocations. It produces an `Artifacts`
//!   which contains the data needed at runtime to find and call Wasm
//!   functions. It is up to the caller to serialize the relevant parts of the
//!   `Artifacts` into the ELF file.

use crate::Engine;
use anyhow::Result;
use std::collections::{btree_map, BTreeMap, BTreeSet};
use std::{any::Any, collections::HashMap};
use wasmtime_environ::{
    Compiler, DefinedFuncIndex, FuncIndex, FunctionBodyData, ModuleTranslation, ModuleType,
    ModuleTypes, PrimaryMap, SignatureIndex, StaticModuleIndex, Tunables, WasmFunctionInfo,
};
use wasmtime_jit::{CompiledFunctionInfo, CompiledModuleInfo};

type CompileInput<'a> =
    Box<dyn FnOnce(&Tunables, &dyn Compiler) -> Result<CompileOutput> + Send + 'a>;

/// A sortable, comparable key for a compilation output.
///
/// Two `u32`s to align with `cranelift_codegen::ir::UserExternalName`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct CompileKey {
    // [ kind:i3 module:i29 ]
    namespace: u32,
    index: u32,
}

impl CompileKey {
    const KIND_MASK: u32 = 0b111 << 29;

    fn kind(&self) -> u32 {
        self.namespace & Self::KIND_MASK
    }

    fn module(&self) -> StaticModuleIndex {
        StaticModuleIndex::from_u32(self.namespace & !Self::KIND_MASK)
    }

    const WASM_FUNCTION_KIND: u32 = 0b000 << 29;
    const ARRAY_TO_WASM_TRAMPOLINE_KIND: u32 = 0b001 << 29;
    const NATIVE_TO_WASM_TRAMPOLINE_KIND: u32 = 0b010 << 29;
    const WASM_TO_NATIVE_TRAMPOLINE_KIND: u32 = 0b011 << 29;
    // NB: more kinds in the other `impl` block.

    fn wasm_function(module: StaticModuleIndex, index: DefinedFuncIndex) -> Self {
        debug_assert_eq!(module.as_u32() & Self::KIND_MASK, 0);
        Self {
            namespace: Self::WASM_FUNCTION_KIND | module.as_u32(),
            index: index.as_u32(),
        }
    }

    fn array_to_wasm_trampoline(module: StaticModuleIndex, index: DefinedFuncIndex) -> Self {
        debug_assert_eq!(module.as_u32() & Self::KIND_MASK, 0);
        Self {
            namespace: Self::ARRAY_TO_WASM_TRAMPOLINE_KIND | module.as_u32(),
            index: index.as_u32(),
        }
    }

    fn native_to_wasm_trampoline(module: StaticModuleIndex, index: DefinedFuncIndex) -> Self {
        debug_assert_eq!(module.as_u32() & Self::KIND_MASK, 0);
        Self {
            namespace: Self::NATIVE_TO_WASM_TRAMPOLINE_KIND | module.as_u32(),
            index: index.as_u32(),
        }
    }

    fn wasm_to_native_trampoline(index: SignatureIndex) -> Self {
        Self {
            namespace: Self::WASM_TO_NATIVE_TRAMPOLINE_KIND,
            index: index.as_u32(),
        }
    }
}

#[cfg(feature = "component-model")]
impl CompileKey {
    const LOWERING_KIND: u32 = 0b100 << 29;
    const ALWAYS_TRAP_KIND: u32 = 0b101 << 29;
    const TRANSCODER_KIND: u32 = 0b110 << 29;

    fn lowering(index: wasmtime_environ::component::LoweredIndex) -> Self {
        Self {
            namespace: Self::LOWERING_KIND,
            index: index.as_u32(),
        }
    }

    fn always_trap(index: wasmtime_environ::component::RuntimeAlwaysTrapIndex) -> Self {
        Self {
            namespace: Self::ALWAYS_TRAP_KIND,
            index: index.as_u32(),
        }
    }

    fn transcoder(index: wasmtime_environ::component::RuntimeTranscoderIndex) -> Self {
        Self {
            namespace: Self::TRANSCODER_KIND,
            index: index.as_u32(),
        }
    }
}

#[derive(Clone, Copy)]
enum CompiledFunction<T> {
    Function(T),
    #[cfg(feature = "component-model")]
    AllCallFunc(wasmtime_environ::component::AllCallFunc<T>),
}

impl<T> CompiledFunction<T> {
    fn unwrap_function(self) -> T {
        match self {
            Self::Function(f) => f,
            #[cfg(feature = "component-model")]
            Self::AllCallFunc(_) => panic!("CompiledFunction::unwrap_function"),
        }
    }

    #[cfg(feature = "component-model")]
    fn unwrap_all_call_func(self) -> wasmtime_environ::component::AllCallFunc<T> {
        match self {
            Self::AllCallFunc(f) => f,
            Self::Function(_) => panic!("CompiledFunction::unwrap_all_call_func"),
        }
    }
}

#[cfg(feature = "component-model")]
impl<T> From<wasmtime_environ::component::AllCallFunc<T>> for CompiledFunction<T> {
    fn from(f: wasmtime_environ::component::AllCallFunc<T>) -> Self {
        Self::AllCallFunc(f)
    }
}

struct CompileOutput {
    key: CompileKey,
    symbol: String,
    function: CompiledFunction<Box<dyn Any + Send>>,
    info: Option<WasmFunctionInfo>,
}

/// The collection of things we need to compile for a Wasm module or component.
pub struct CompileInputs<'a> {
    inputs: Vec<CompileInput<'a>>,
}

fn push_input<'a, 'data>(
    inputs: &mut Vec<CompileInput<'a>>,
    f: impl FnOnce(&Tunables, &dyn Compiler) -> Result<CompileOutput> + Send + 'a,
) {
    inputs.push(Box::new(f) as _);
}

impl<'a> CompileInputs<'a> {
    /// Create the `CompileInputs` for a core Wasm module.
    pub fn for_module(
        types: &'a ModuleTypes,
        translation: &'a ModuleTranslation<'a>,
        functions: PrimaryMap<DefinedFuncIndex, FunctionBodyData<'a>>,
    ) -> Self {
        let mut inputs = vec![];
        let module_index = StaticModuleIndex::from_u32(0);

        Self::collect_inputs_in_translations(
            types,
            [(module_index, translation, functions)],
            &mut inputs,
        );

        CompileInputs { inputs }
    }

    /// Create a `CompileInputs` for a component.
    #[cfg(feature = "component-model")]
    pub fn for_component(
        types: &'a wasmtime_environ::component::ComponentTypes,
        component: &'a wasmtime_environ::component::Component,
        module_translations: impl IntoIterator<
            Item = (
                StaticModuleIndex,
                &'a ModuleTranslation<'a>,
                PrimaryMap<DefinedFuncIndex, FunctionBodyData<'a>>,
            ),
        >,
    ) -> Self {
        let mut inputs = vec![];

        Self::collect_inputs_in_translations(
            types.module_types(),
            module_translations,
            &mut inputs,
        );

        for init in &component.initializers {
            match init {
                wasmtime_environ::component::GlobalInitializer::AlwaysTrap(always_trap) => {
                    push_input(&mut inputs, move |_tunables, compiler| {
                        Ok(CompileOutput {
                            key: CompileKey::always_trap(always_trap.index),
                            symbol: always_trap.symbol_name(),
                            function: compiler
                                .component_compiler()
                                .compile_always_trap(&types[always_trap.canonical_abi])?
                                .into(),
                            info: None,
                        })
                    });
                }
                wasmtime_environ::component::GlobalInitializer::Transcoder(transcoder) => {
                    push_input(&mut inputs, move |_tunables, compiler| {
                        Ok(CompileOutput {
                            key: CompileKey::transcoder(transcoder.index),
                            symbol: transcoder.symbol_name(),
                            function: compiler
                                .component_compiler()
                                .compile_transcoder(component, transcoder, types)?
                                .into(),
                            info: None,
                        })
                    });
                }
                wasmtime_environ::component::GlobalInitializer::LowerImport(lower_import) => {
                    push_input(&mut inputs, move |_tunables, compiler| {
                        Ok(CompileOutput {
                            key: CompileKey::lowering(lower_import.index),
                            symbol: lower_import.symbol_name(),
                            function: compiler
                                .component_compiler()
                                .compile_lowered_trampoline(component, lower_import, types)?
                                .into(),
                            info: None,
                        })
                    });
                }
                wasmtime_environ::component::GlobalInitializer::InstantiateModule(_)
                | wasmtime_environ::component::GlobalInitializer::ExtractMemory(_)
                | wasmtime_environ::component::GlobalInitializer::ExtractRealloc(_)
                | wasmtime_environ::component::GlobalInitializer::ExtractPostReturn(_) => {
                    // Nothing to compile for these.
                }
            }
        }

        CompileInputs { inputs }
    }

    fn collect_inputs_in_translations(
        types: &'a ModuleTypes,
        translations: impl IntoIterator<
            Item = (
                StaticModuleIndex,
                &'a ModuleTranslation<'a>,
                PrimaryMap<DefinedFuncIndex, FunctionBodyData<'a>>,
            ),
        >,
        inputs: &mut Vec<CompileInput<'a>>,
    ) {
        let mut sigs = BTreeMap::new();

        for (module, translation, functions) in translations {
            for (def_func_index, func_body) in functions {
                push_input(inputs, move |tunables, compiler| {
                    let func_index = translation.module.func_index(def_func_index);
                    let (info, function) = compiler.compile_function(
                        translation,
                        def_func_index,
                        func_body,
                        tunables,
                        types,
                    )?;
                    Ok(CompileOutput {
                        key: CompileKey::wasm_function(module, def_func_index),
                        symbol: format!(
                            "wasm[{}]::function[{}]",
                            module.as_u32(),
                            func_index.as_u32()
                        ),
                        function: CompiledFunction::Function(function),
                        info: Some(info),
                    })
                });

                let func_index = translation.module.func_index(def_func_index);
                if translation.module.functions[func_index].is_escaping() {
                    push_input(inputs, move |_tunables, compiler| {
                        let func_index = translation.module.func_index(def_func_index);
                        let trampoline = compiler.compile_array_to_wasm_trampoline(
                            translation,
                            types,
                            def_func_index,
                        )?;
                        Ok(CompileOutput {
                            key: CompileKey::array_to_wasm_trampoline(module, def_func_index),
                            symbol: format!(
                                "wasm[{}]::array_to_wasm_trampoline[{}]",
                                module.as_u32(),
                                func_index.as_u32()
                            ),
                            function: CompiledFunction::Function(trampoline),
                            info: None,
                        })
                    });

                    push_input(inputs, move |_tunables, compiler| {
                        let func_index = translation.module.func_index(def_func_index);
                        let trampoline = compiler.compile_native_to_wasm_trampoline(
                            translation,
                            types,
                            def_func_index,
                        )?;
                        Ok(CompileOutput {
                            key: CompileKey::native_to_wasm_trampoline(module, def_func_index),
                            symbol: format!(
                                "wasm[{}]::native_to_wasm_trampoline[{}]",
                                module.as_u32(),
                                func_index.as_u32()
                            ),
                            function: CompiledFunction::Function(trampoline),
                            info: None,
                        })
                    });
                }
            }

            sigs.extend(translation.module.types.iter().map(|(_, ty)| match ty {
                ModuleType::Function(ty) | ModuleType::Continuation(ty) => (*ty, translation),
            }));
        }

        for (signature, translation) in sigs {
            push_input(inputs, move |_tunables, compiler| {
                let wasm_func_ty = &types[signature];
                let trampoline =
                    compiler.compile_wasm_to_native_trampoline(translation, wasm_func_ty)?;
                Ok(CompileOutput {
                    key: CompileKey::wasm_to_native_trampoline(signature),
                    symbol: format!(
                        "signatures[{}]::wasm_to_native_trampoline",
                        signature.as_u32()
                    ),
                    function: CompiledFunction::Function(trampoline),
                    info: None,
                })
            });
        }
    }

    /// Compile these `CompileInput`s (maybe in parallel) and return the
    /// resulting `UnlinkedCompileOutput`s.
    pub fn compile(self, engine: &Engine) -> Result<UnlinkedCompileOutputs> {
        let tunables = &engine.config().tunables;
        let compiler = engine.compiler();

        // Compile each individual input in parallel.
        let raw_outputs = engine.run_maybe_parallel(self.inputs, |f| f(tunables, compiler))?;

        // Bucket the outputs by kind.
        let mut outputs: BTreeMap<u32, Vec<CompileOutput>> = BTreeMap::new();
        for output in raw_outputs {
            outputs.entry(output.key.kind()).or_default().push(output);
        }

        // Assert that the elements within a bucket are all sorted as we expect
        // them to be.
        fn is_sorted_by_key<T, K>(items: &[T], f: impl Fn(&T) -> K) -> bool
        where
            K: PartialOrd,
        {
            items
                .windows(2)
                .all(|window| f(&window[0]) <= f(&window[1]))
        }
        debug_assert!(outputs
            .values()
            .all(|funcs| is_sorted_by_key(funcs, |x| x.key)));

        Ok(UnlinkedCompileOutputs { outputs })
    }
}

#[derive(Default)]
pub struct UnlinkedCompileOutputs {
    // A map from kind to `CompileOutput`.
    outputs: BTreeMap<u32, Vec<CompileOutput>>,
}

impl UnlinkedCompileOutputs {
    /// Flatten all our functions into a single list and remember each of their
    /// indices within it.
    pub fn pre_link(self) -> (Vec<(String, Box<dyn Any + Send>)>, FunctionIndices) {
        // The order the functions end up within `compiled_funcs` is the order
        // that they will be laid out in the ELF file, so try and group hot and
        // cold functions together as best we can. However, because we bucket by
        // kind, we shouldn't have any issues with, e.g., cold trampolines
        // appearing in between hot Wasm functions.
        let mut compiled_funcs = vec![];
        let mut indices = FunctionIndices::default();
        for x in self.outputs.into_iter().flat_map(|(_kind, xs)| xs) {
            let index = match x.function {
                CompiledFunction::Function(f) => {
                    let index = compiled_funcs.len();
                    compiled_funcs.push((x.symbol, f));
                    CompiledFunction::Function(index)
                }
                #[cfg(feature = "component-model")]
                CompiledFunction::AllCallFunc(f) => {
                    let array_call = compiled_funcs.len();
                    compiled_funcs.push((format!("{}_array_call", x.symbol), f.array_call));
                    let native_call = compiled_funcs.len();
                    compiled_funcs.push((format!("{}_native_call", x.symbol), f.native_call));
                    let wasm_call = compiled_funcs.len();
                    compiled_funcs.push((format!("{}_wasm_call", x.symbol), f.wasm_call));
                    CompiledFunction::AllCallFunc(wasmtime_environ::component::AllCallFunc {
                        array_call,
                        native_call,
                        wasm_call,
                    })
                }
            };

            if x.key.kind() == CompileKey::WASM_FUNCTION_KIND
                || x.key.kind() == CompileKey::ARRAY_TO_WASM_TRAMPOLINE_KIND
                || x.key.kind() == CompileKey::NATIVE_TO_WASM_TRAMPOLINE_KIND
            {
                indices
                    .compiled_func_index_to_module
                    .insert(index.unwrap_function(), x.key.module());
                if let Some(info) = x.info {
                    indices.wasm_function_infos.insert(x.key, info);
                }
            }

            indices
                .indices
                .entry(x.key.kind())
                .or_default()
                .insert(x.key, index);
        }
        (compiled_funcs, indices)
    }
}

#[derive(Default)]
pub struct FunctionIndices {
    // A reverse map from an index in `compiled_funcs` to the
    // `StaticModuleIndex` for that function.
    compiled_func_index_to_module: HashMap<usize, StaticModuleIndex>,

    // A map from Wasm functions' compile keys to their infos.
    wasm_function_infos: HashMap<CompileKey, WasmFunctionInfo>,

    // The index of each compiled function, bucketed by compile key kind.
    indices: BTreeMap<u32, BTreeMap<CompileKey, CompiledFunction<usize>>>,
}

impl FunctionIndices {
    /// Link the compiled functions together, resolving relocations, and append
    /// them to the given ELF file.
    pub fn link_and_append_code<'a>(
        mut self,
        mut obj: object::write::Object<'static>,
        tunables: &'a Tunables,
        compiler: &dyn Compiler,
        compiled_funcs: Vec<(String, Box<dyn Any + Send>)>,
        translations: PrimaryMap<StaticModuleIndex, ModuleTranslation<'_>>,
    ) -> Result<(wasmtime_jit::ObjectBuilder<'a>, Artifacts)> {
        // Append all the functions to the ELF file.
        //
        // The result is a vector parallel to `compiled_funcs` where
        // `symbol_ids_and_locs[i]` is the symbol ID and function location of
        // `compiled_funcs[i]`.
        let symbol_ids_and_locs = compiler.append_code(
            &mut obj,
            &compiled_funcs,
            tunables,
            &|caller_index: usize, callee_index: FuncIndex| {
                let module = self
                    .compiled_func_index_to_module
                    .get(&caller_index)
                    .copied()
                    .expect("should only reloc inside wasm function callers");
                let def_func_index = translations[module]
                    .module
                    .defined_func_index(callee_index)
                    .unwrap();
                self.indices[&CompileKey::WASM_FUNCTION_KIND]
                    [&CompileKey::wasm_function(module, def_func_index)]
                    .unwrap_function()
            },
        )?;

        // If requested, generate and add DWARF information.
        if tunables.generate_native_debuginfo &&
            // We can only add DWARF once. Supporting DWARF for components of
            // multiple Wasm modules will require merging the DWARF sections
            // together.
            translations.len() == 1
        {
            for (module, translation) in &translations {
                let funcs: PrimaryMap<_, _> = self
                    .indices
                    .get(&CompileKey::WASM_FUNCTION_KIND)
                    .map(|xs| {
                        xs.range(
                            CompileKey::wasm_function(module, DefinedFuncIndex::from_u32(0))
                                ..=CompileKey::wasm_function(
                                    module,
                                    DefinedFuncIndex::from_u32(u32::MAX - 1),
                                ),
                        )
                    })
                    .into_iter()
                    .flat_map(|x| x)
                    .map(|(_, x)| {
                        let i = x.unwrap_function();
                        (symbol_ids_and_locs[i].0, &*compiled_funcs[i].1)
                    })
                    .collect();
                if !funcs.is_empty() {
                    compiler.append_dwarf(&mut obj, translation, &funcs)?;
                }
            }
        }

        let mut obj = wasmtime_jit::ObjectBuilder::new(obj, tunables);
        let mut artifacts = Artifacts::default();

        // Finally, build our binary artifacts that map things like `FuncIndex`
        // to a function location and all of that using the indices we saved
        // earlier and the function locations we just received after appending
        // the code.

        let mut wasm_functions = self
            .indices
            .remove(&CompileKey::WASM_FUNCTION_KIND)
            .unwrap_or_default()
            .into_iter()
            .peekable();

        fn wasm_functions_for_module(
            wasm_functions: &mut std::iter::Peekable<
                btree_map::IntoIter<CompileKey, CompiledFunction<usize>>,
            >,
            module: StaticModuleIndex,
        ) -> impl Iterator<Item = (CompileKey, CompiledFunction<usize>)> + '_ {
            std::iter::from_fn(move || {
                let (key, _) = wasm_functions.peek()?;
                if key.module() == module {
                    wasm_functions.next()
                } else {
                    None
                }
            })
        }

        let mut array_to_wasm_trampolines = self
            .indices
            .remove(&CompileKey::ARRAY_TO_WASM_TRAMPOLINE_KIND)
            .unwrap_or_default();

        let mut native_to_wasm_trampolines = self
            .indices
            .remove(&CompileKey::NATIVE_TO_WASM_TRAMPOLINE_KIND)
            .unwrap_or_default();

        // NB: unlike the above maps this is not emptied out during iteration
        // since each module may reach into different portions of this map.
        let wasm_to_native_trampolines = self
            .indices
            .remove(&CompileKey::WASM_TO_NATIVE_TRAMPOLINE_KIND)
            .unwrap_or_default();

        artifacts.modules = translations
            .into_iter()
            .map(|(module, translation)| {
                let funcs: PrimaryMap<DefinedFuncIndex, CompiledFunctionInfo> =
                    wasm_functions_for_module(&mut wasm_functions, module)
                        .map(|(key, wasm_func_index)| {
                            let wasm_func_index = wasm_func_index.unwrap_function();
                            let wasm_func_loc = symbol_ids_and_locs[wasm_func_index].1;
                            let wasm_func_info = self.wasm_function_infos.remove(&key).unwrap();

                            let array_to_wasm_trampoline = array_to_wasm_trampolines
                                .remove(&CompileKey::array_to_wasm_trampoline(
                                    key.module(),
                                    DefinedFuncIndex::from_u32(key.index),
                                ))
                                .map(|x| symbol_ids_and_locs[x.unwrap_function()].1);

                            let native_to_wasm_trampoline = native_to_wasm_trampolines
                                .remove(&CompileKey::native_to_wasm_trampoline(
                                    key.module(),
                                    DefinedFuncIndex::from_u32(key.index),
                                ))
                                .map(|x| symbol_ids_and_locs[x.unwrap_function()].1);

                            CompiledFunctionInfo::new(
                                wasm_func_info,
                                wasm_func_loc,
                                array_to_wasm_trampoline,
                                native_to_wasm_trampoline,
                            )
                        })
                        .collect();

                let unique_and_sorted_sigs = translation
                    .module
                    .types
                    .iter()
                    .map(|(_, ty)| match ty {
                        ModuleType::Function(ty) | ModuleType::Continuation(ty) => *ty,
                    })
                    .collect::<BTreeSet<_>>();
                let wasm_to_native_trampolines = unique_and_sorted_sigs
                    .iter()
                    .map(|idx| {
                        let key = CompileKey::wasm_to_native_trampoline(*idx);
                        let compiled = wasm_to_native_trampolines[&key];
                        (*idx, symbol_ids_and_locs[compiled.unwrap_function()].1)
                    })
                    .collect();

                obj.append(translation, funcs, wasm_to_native_trampolines)
            })
            .collect::<Result<PrimaryMap<_, _>>>()?;

        #[cfg(feature = "component-model")]
        {
            artifacts.lowerings = self
                .indices
                .remove(&CompileKey::LOWERING_KIND)
                .unwrap_or_default()
                .into_iter()
                .map(|(_id, x)| x.unwrap_all_call_func().map(|i| symbol_ids_and_locs[i].1))
                .collect();
            artifacts.transcoders = self
                .indices
                .remove(&CompileKey::TRANSCODER_KIND)
                .unwrap_or_default()
                .into_iter()
                .map(|(_id, x)| x.unwrap_all_call_func().map(|i| symbol_ids_and_locs[i].1))
                .collect();
            artifacts.always_traps = self
                .indices
                .remove(&CompileKey::ALWAYS_TRAP_KIND)
                .unwrap_or_default()
                .into_iter()
                .map(|(_id, x)| x.unwrap_all_call_func().map(|i| symbol_ids_and_locs[i].1))
                .collect();
        }

        debug_assert!(
            self.indices.is_empty(),
            "Should have processed all compile outputs"
        );

        Ok((obj, artifacts))
    }
}

/// The artifacts necessary for finding and calling Wasm functions at runtime,
/// to be serialized into an ELF file.
#[derive(Default)]
pub struct Artifacts {
    pub modules: PrimaryMap<StaticModuleIndex, CompiledModuleInfo>,
    #[cfg(feature = "component-model")]
    pub lowerings: PrimaryMap<
        wasmtime_environ::component::LoweredIndex,
        wasmtime_environ::component::AllCallFunc<wasmtime_environ::FunctionLoc>,
    >,
    #[cfg(feature = "component-model")]
    pub always_traps: PrimaryMap<
        wasmtime_environ::component::RuntimeAlwaysTrapIndex,
        wasmtime_environ::component::AllCallFunc<wasmtime_environ::FunctionLoc>,
    >,
    #[cfg(feature = "component-model")]
    pub transcoders: PrimaryMap<
        wasmtime_environ::component::RuntimeTranscoderIndex,
        wasmtime_environ::component::AllCallFunc<wasmtime_environ::FunctionLoc>,
    >,
}

impl Artifacts {
    /// Assuming this compilation was for a single core Wasm module, get the
    /// resulting `CompiledModuleInfo`.
    pub fn unwrap_as_module_info(self) -> CompiledModuleInfo {
        assert_eq!(self.modules.len(), 1);
        #[cfg(feature = "component-model")]
        {
            assert!(self.lowerings.is_empty());
            assert!(self.always_traps.is_empty());
            assert!(self.transcoders.is_empty());
        }
        self.modules.into_iter().next().unwrap().1
    }
}
