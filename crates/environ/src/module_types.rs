use crate::{
    Module, ModuleType, PrimaryMap, SignatureIndex, TypeConvert, TypeIndex, WasmContType, WasmFuncType,
    WasmHeapType,
};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ops::Index;
use wasmparser::types::CoreTypeId;
use wasmparser::UnpackedIndex;

/// All types used in a core wasm module.
///
/// At this time this only contains function types. Note, though, that function
/// types are deduplicated within this [`ModuleTypes`].
///
/// Note that accesing this type is primarily done through the `Index`
/// implementations for this type.
#[derive(Default, Serialize, Deserialize)]
#[allow(missing_docs)]
pub struct ModuleTypes {
    wasm_signatures: PrimaryMap<SignatureIndex, WasmFuncType>,
}

impl ModuleTypes {
    /// Returns an iterator over all the wasm function signatures found within
    /// this module.
    pub fn wasm_signatures(&self) -> impl Iterator<Item = (SignatureIndex, &WasmFuncType)> {
        self.wasm_signatures.iter()
    }
}

impl Index<SignatureIndex> for ModuleTypes {
    type Output = WasmFuncType;

    fn index(&self, sig: SignatureIndex) -> &WasmFuncType {
        &self.wasm_signatures[sig]
    }
}

/// A builder for [`ModuleTypes`].
#[derive(Default)]
#[allow(missing_docs)]
pub struct ModuleTypesBuilder {
    types: ModuleTypes,
    interned_func_types: HashMap<WasmFuncType, SignatureIndex>,
    interned_cont_types: HashMap<WasmContType, SignatureIndex>,
    wasmparser_to_wasmtime: HashMap<CoreTypeId, SignatureIndex>,
}

impl ModuleTypesBuilder {
    /// Reserves space for `amt` more type signatures.
    pub fn reserve_wasm_signatures(&mut self, amt: usize) {
        self.types.wasm_signatures.reserve(amt);
    }

    /// Interns the `sig` specified and returns a unique `SignatureIndex` that
    /// can be looked up within [`ModuleTypes`] to recover the [`WasmFuncType`]
    /// at runtime.
    pub fn wasm_func_type(&mut self, id: CoreTypeId, sig: WasmFuncType) -> SignatureIndex {
        let sig = self.intern_func_type(sig);
        self.wasmparser_to_wasmtime.insert(id, sig);
        sig
    }

    /// Returns a unique `SignatureIndex` that can be looked up within
    /// [`ModuleTypes`] to recover the [`WasmContType`] at runtime.
    pub fn wasm_cont_type(&mut self, id: CoreTypeId, sig: WasmContType) -> SignatureIndex {
        // TODO(dhil): Continuation types should be interned
        // like function types... I believe the necessary
        // infrastructure to support this change will come
        // from upstream when they implement array and struct.
        let sig_index = self.intern_cont_type(sig);
        self.wasmparser_to_wasmtime.insert(id, sig_index);
        sig_index
    }

    fn intern_cont_type(&mut self, sig: WasmContType) -> SignatureIndex {
        if let Some(idx) = self.interned_cont_types.get(&sig) {
            return *idx;
        }

        let idx = WasmContType::signature_index(sig.clone());
        self.interned_cont_types.insert(sig, idx);
        return idx;
    }


    fn intern_func_type(&mut self, sig: WasmFuncType) -> SignatureIndex {
        if let Some(idx) = self.interned_func_types.get(&sig) {
            return *idx;
        }

        let idx = self.types.wasm_signatures.push(sig.clone());
        self.interned_func_types.insert(sig, idx);
        return idx;
    }

    /// Returns the result [`ModuleTypes`] of this builder.
    pub fn finish(self) -> ModuleTypes {
        self.types
    }

    /// Returns an iterator over all the wasm function signatures found within
    /// this module.
    pub fn wasm_signatures(&self) -> impl Iterator<Item = (SignatureIndex, &WasmFuncType)> {
        self.types.wasm_signatures()
    }
}

// Forward the indexing impl to the internal `ModuleTypes`
impl<T> Index<T> for ModuleTypesBuilder
where
    ModuleTypes: Index<T>,
{
    type Output = <ModuleTypes as Index<T>>::Output;

    fn index(&self, sig: T) -> &Self::Output {
        &self.types[sig]
    }
}

#[allow(missing_docs)]
pub struct WasmparserTypeConverter<'a> {
    pub types: &'a ModuleTypesBuilder,
    pub module: &'a Module,
}

impl TypeConvert for WasmparserTypeConverter<'_> {
    fn lookup_heap_type(&self, index: UnpackedIndex) -> WasmHeapType {
        match index {
            UnpackedIndex::Id(id) => {
                let signature = self.types.wasmparser_to_wasmtime[&id];
                WasmHeapType::TypedFunc(signature)
            }
            UnpackedIndex::RecGroup(_) => unreachable!(),
            UnpackedIndex::Module(i) => {
                let i = TypeIndex::from_u32(i);
                match self.module.types[i] {
                    ModuleType::Function(sig) => WasmHeapType::TypedFunc(sig),
                    ModuleType::Continuation(sig) => WasmHeapType::TypedFunc(sig), // TODO(dhil): ehh
                }
            }
        }
    }
}
