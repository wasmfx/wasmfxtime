use super::shared;

use cranelift_codegen::ir;
use cranelift_codegen::ir::condcodes::*;
use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::InstBuilder;
use cranelift_frontend::{FunctionBuilder, Switch};
use cranelift_wasm::FuncEnvironment;
use cranelift_wasm::{FuncTranslationState, WasmResult, WasmValType};
use wasmtime_environ::{BuiltinFunctionIndex, PtrSize};

#[allow(unused_imports)]
pub(crate) use shared::typed_continuations_cont_ref_get_cont_obj;
#[allow(unused_imports)]
pub(crate) use shared::typed_continuations_new_cont_ref;

#[macro_use]
pub(crate) mod typed_continuation_helpers {
    use cranelift_codegen::ir;
    use cranelift_codegen::ir::condcodes::IntCC;
    use cranelift_codegen::ir::types::*;
    use cranelift_codegen::ir::InstBuilder;
    use cranelift_frontend::FunctionBuilder;
    use std::mem;
    use wasmtime_environ::BuiltinFunctionIndex;
    use wasmtime_environ::PtrSize;

    // This is a reference to this very module.
    // We need it so that we can refer to the functions inside this module from
    // macros, such that the same path works when the macro is expanded inside
    // or outside of this module.
    use crate::wasmfx::optimized::typed_continuation_helpers as tc;

    /// Low-level implementation of debug printing. Do not use directly; see
    /// `emit_debug_println!` macro for doing actual printing.
    ///
    /// Takes a string literal which may contain placeholders similarly to those
    /// supported by `std::fmt`.
    ///
    /// Currently supported placeholders:
    /// {}       for unsigned integers
    /// {:p}     for printing pointers (in hex form)
    ///
    /// When printing, we replace them with the corresponding values in `vals`.
    /// Thus, the number of placeholders in `s` must match the number of entries
    /// in `vals`.
    pub fn emit_debug_print<'a>(
        env: &mut crate::func_environ::FuncEnvironment<'a>,
        builder: &mut FunctionBuilder,
        s: &'static str,
        vals: &[ir::Value],
    ) {
        let print_s_infix = |env: &mut crate::func_environ::FuncEnvironment<'a>,
                             builder: &mut FunctionBuilder,
                             start: usize,
                             end: usize| {
            if start < end {
                let s: &'static str = &s[start..end];
                // This is quite dodgy, which is why we can only do this for
                // debugging purposes:
                // At jit time, we take a pointer to the slice of the (static)
                // string, thus yielding an address within wasmtime's DATA
                // section. This pointer is hard-code into generated code. We do
                // not emit any kind of relocation information, which means that
                // this breaks if we were to store the generated code and use it
                // during subsequent executions of wasmtime (e.g., when using
                // wasmtime compile).
                let ptr = s.as_ptr();
                let ptr = builder.ins().iconst(env.pointer_type(), ptr as i64);
                let len = s.len();
                let len = builder.ins().iconst(I64, len as i64);

                let index = BuiltinFunctionIndex::tc_print_str();
                let sig = env.builtin_function_signatures.tc_print_str(builder.func);
                env.generate_builtin_call_no_return_val(builder, index, sig, vec![ptr, len]);
            }
        };
        let print_int = |env: &mut crate::func_environ::FuncEnvironment<'a>,
                         builder: &mut FunctionBuilder,
                         val: ir::Value| {
            let index = BuiltinFunctionIndex::tc_print_int();
            let sig = env.builtin_function_signatures.tc_print_int(builder.func);
            let ty = builder.func.dfg.value_type(val);
            let val = match ty {
                I32 => builder.ins().uextend(I64, val),
                I64 => val,
                _ => panic!("Cannot print type {}", ty),
            };
            env.generate_builtin_call_no_return_val(builder, index, sig, vec![val]);
        };
        let print_pointer = |env: &mut crate::func_environ::FuncEnvironment<'a>,
                             builder: &mut FunctionBuilder,
                             ptr: ir::Value| {
            let index = BuiltinFunctionIndex::tc_print_pointer();
            let sig = env
                .builtin_function_signatures
                .tc_print_pointer(builder.func);
            env.generate_builtin_call_no_return_val(builder, index, sig, vec![ptr]);
        };

        if wasmtime_continuations::ENABLE_DEBUG_PRINTING {
            let mut prev_end = 0;
            let mut i = 0;

            let mut ph_matches: Vec<(usize, &'static str)> = s
                .match_indices("{}")
                .chain(s.match_indices("{:p}"))
                .collect();
            ph_matches.sort_by_key(|(index, _)| *index);

            for (start, matched_ph) in ph_matches {
                let end = start + matched_ph.len();

                assert!(
                    i < vals.len(),
                    "Must supply as many entries in vals as there are placeholders in the string"
                );

                print_s_infix(env, builder, prev_end, start);
                match matched_ph {
                    "{}" => print_int(env, builder, vals[i]),
                    "{:p}" => print_pointer(env, builder, vals[i]),
                    u => panic!("Unsupported placeholder in debug_print input string: {}", u),
                }
                prev_end = end;
                i += 1;
            }
            assert_eq!(
                i,
                vals.len(),
                "Must supply as many entries in vals as there are placeholders in the string"
            );

            print_s_infix(env, builder, prev_end, s.len());
        }
    }

    /// Emits code to print debug information. Only actually prints in debug
    /// builds and if debug printing flag is enabled. The third and all
    /// following arguments are like those to println!: A string literal with
    /// placeholders followed by the actual values.
    ///
    /// Summary of arguments:
    /// * `env` - Type &mut crate::func_environ::FuncEnvironment<'a>
    /// * `builder` - Type &mut FunctionBuilder,
    /// * `msg` : String literal, containing placeholders like those supported by println!
    /// * remaining arguments: ir::Values filled into the placeholders in `msg`
    #[allow(unused_macros)]
    macro_rules! emit_debug_println {
        ($env : expr, $builder : expr, $msg : literal, $( $arg:expr ),*) => {
            let msg_newline : &'static str= std::concat!(
                $msg,
                "\n"
            );
            tc::emit_debug_print($env, $builder, msg_newline, &[$($arg),*]);
        }
    }

    /// Low-level implementation of assertion mechanism. Use emit_debug_* macros
    /// instead.
    ///
    /// If `ENABLE_DEBUG_PRINTING` is enabled, `error_str` is printed before
    /// trapping in case of an assertion violation.
    pub fn emit_debug_assert_generic<'a>(
        env: &mut crate::func_environ::FuncEnvironment<'a>,
        builder: &mut FunctionBuilder,
        condition: ir::Value,
        error_str: &'static str,
    ) {
        if cfg!(debug_assertions) {
            if wasmtime_continuations::ENABLE_DEBUG_PRINTING {
                let failure_block = builder.create_block();
                let continue_block = builder.create_block();

                builder
                    .ins()
                    .brif(condition, continue_block, &[], failure_block, &[]);

                builder.switch_to_block(failure_block);
                builder.seal_block(failure_block);

                emit_debug_print(env, builder, error_str, &[]);
                builder.ins().debugtrap();
                builder.ins().jump(continue_block, &[]);

                builder.switch_to_block(continue_block);
                builder.seal_block(continue_block);
            } else {
                builder
                    .ins()
                    .trapz(condition, ir::TrapCode::User(crate::DEBUG_ASSERT_TRAP_CODE));
            }
        }
    }

    /// Low-level implementation of assertion mechanism. Use emit_debug_* macros
    /// instead.
    ///
    /// If `ENABLE_DEBUG_PRINTING` is enabled, `error_str` is printed before
    /// trapping in case of an assertion violation. Here, `error_str` is expected
    /// to contain two placeholders, such as {} or {:p}, which are replaced with
    /// `v1` and `v2` when printing.
    pub fn emit_debug_assert_icmp<'a>(
        env: &mut crate::func_environ::FuncEnvironment<'a>,
        builder: &mut FunctionBuilder,
        operator: IntCC,
        v1: ir::Value,
        v2: ir::Value,
        error_str: &'static str,
    ) {
        if cfg!(debug_assertions) {
            let cmp_res = builder.ins().icmp(operator, v1, v2);

            if wasmtime_continuations::ENABLE_DEBUG_PRINTING {
                let failure_block = builder.create_block();
                let continue_block = builder.create_block();

                builder
                    .ins()
                    .brif(cmp_res, continue_block, &[], failure_block, &[]);

                builder.switch_to_block(failure_block);
                builder.seal_block(failure_block);

                emit_debug_print(env, builder, error_str, &[v1, v2]);
                builder.ins().debugtrap();
                builder.ins().jump(continue_block, &[]);

                builder.switch_to_block(continue_block);
                builder.seal_block(continue_block);
            } else {
                builder
                    .ins()
                    .trapz(cmp_res, ir::TrapCode::User(crate::DEBUG_ASSERT_TRAP_CODE));
            }
        }
    }

    /// Used to implement other macros, do not use directly.
    macro_rules! emit_debug_assert_icmp {
        ( $env : expr,
            $builder: expr,
        $operator : expr,
        $operator_string  : expr,
        $v1 : expr,
        $v2 : expr) => {
            let msg: &'static str = std::concat!(
                "assertion failure in ",
                std::file!(),
                ", line ",
                std::line!(),
                ": {} ",
                $operator_string,
                " {} does not hold\n"
            );
            tc::emit_debug_assert_icmp($env, $builder, $operator, $v1, $v2, msg);
        };
    }

    macro_rules! emit_debug_assert {
        ($env: expr, $builder: expr, $condition: expr) => {
            let msg: &'static str = std::concat!(
                "assertion failure in ",
                std::file!(),
                ", line ",
                std::line!(),
                "\n"
            );
            tc::emit_debug_assert_generic($env, $builder, $condition, msg);
        };
    }

    macro_rules! emit_debug_assert_eq {
        ($env: expr, $builder: expr, $v1 : expr, $v2: expr) => {
            emit_debug_assert_icmp!($env, $builder, IntCC::Equal, "==", $v1, $v2);
        };
    }

    macro_rules! emit_debug_assert_ne {
        ($env: expr, $builder: expr, $v1 : expr, $v2: expr) => {
            emit_debug_assert_icmp!($env, $builder, IntCC::NotEqual, "!=", $v1, $v2);
        };
    }

    macro_rules! emit_debug_assert_ule {
        ($env: expr, $builder: expr, $v1 : expr, $v2: expr) => {
            emit_debug_assert_icmp!(
                $env,
                $builder,
                IntCC::UnsignedLessThanOrEqual,
                "<=",
                $v1,
                $v2
            );
        };
    }

    #[derive(Copy, Clone)]
    pub struct ContinuationObject {
        address: ir::Value,
        pointer_type: ir::Type,
    }

    #[derive(Copy, Clone)]
    pub struct Payloads {
        /// Base address of this object, which must be shifted by `offset` below.
        base: ir::Value,

        /// Adding this (statically) known offset gets us the overall address.
        offset: i32,

        pointer_type: ir::Type,
    }

    #[derive(Copy, Clone)]
    pub struct VMContext {
        address: ir::Value,
        pointer_type: ir::Type,
    }

    /// Size of `wasmtime_continuations::StackChain` in machine words.
    /// Used to verify that we have not changed its representation.
    const STACK_CHAIN_POINTER_COUNT: usize =
        std::mem::size_of::<wasmtime_continuations::StackChain>() / std::mem::size_of::<usize>();

    /// Compile-time representation of wasmtime_continuations::StackChain,
    /// consisting of two `ir::Value`s.
    pub struct StackChain {
        discriminant: ir::Value,
        payload: ir::Value,
        pointer_type: ir::Type,
    }

    impl ContinuationObject {
        pub fn new(address: ir::Value, pointer_type: ir::Type) -> ContinuationObject {
            ContinuationObject {
                address,
                pointer_type,
            }
        }

        pub fn args(&self) -> Payloads {
            let offset = wasmtime_continuations::offsets::continuation_object::ARGS;
            Payloads::new(self.address, offset, self.pointer_type)
        }

        pub fn tag_return_values(&self) -> Payloads {
            let offset = wasmtime_continuations::offsets::continuation_object::TAG_RETURN_VALUES;
            Payloads::new(self.address, offset, self.pointer_type)
        }

        /// Loads the value of the `state` field of the continuation object,
        /// which is represented using the `State` enum.
        fn load_state(&self, builder: &mut FunctionBuilder) -> ir::Value {
            let mem_flags = ir::MemFlags::trusted();
            let offset = wasmtime_continuations::offsets::continuation_object::STATE;

            // Let's make sure that we still represent the State enum as i32.
            debug_assert!(mem::size_of::<wasmtime_continuations::State>() == mem::size_of::<i32>());

            builder.ins().load(I32, mem_flags, self.address, offset)
        }

        /// Sets the value of the `state` field of the continuation object,
        pub fn set_state(
            &self,
            builder: &mut FunctionBuilder,
            state: wasmtime_continuations::State,
        ) {
            let mem_flags = ir::MemFlags::trusted();
            let offset = wasmtime_continuations::offsets::continuation_object::STATE;

            // Let's make sure that we still represent the State enum as i32.
            debug_assert!(mem::size_of::<wasmtime_continuations::State>() == mem::size_of::<i32>());

            let v = builder.ins().iconst(I32, state.discriminant() as i64);
            builder.ins().store(mem_flags, v, self.address, offset);
        }

        /// Checks whether the continuation object is invoked (i.e., `resume`
        /// was called at least once on the object).
        pub fn is_invoked(&self, builder: &mut FunctionBuilder) -> ir::Value {
            // TODO(frank-emrich) In the future, we may get rid of the State field
            // in `ContinuationObject` and try to infer the state by other means.
            // For example, we may alllocate the `ContinuationFiber` lazily, doing
            // so only at the point when a continuation is actualy invoked, meaning
            // that we can use the null-ness of the `fiber` field as an indicator
            // for invokedness.
            let actual_state = self.load_state(builder);
            let invoked: i32 = i32::from(wasmtime_continuations::State::Invoked);
            builder
                .ins()
                .icmp_imm(IntCC::Equal, actual_state, invoked as i64)
        }

        /// Checks whether the continuation object has returned (i.e., the
        /// function used as continuation has returned normally).
        pub fn has_returned(&self, builder: &mut FunctionBuilder) -> ir::Value {
            let actual_state = self.load_state(builder);
            let returned: i32 = i32::from(wasmtime_continuations::State::Returned);
            builder
                .ins()
                .icmp_imm(IntCC::Equal, actual_state, returned as i64)
        }

        /// Returns pointer to buffer where results are stored after a
        /// continuation has returned.
        pub fn get_results<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
        ) -> ir::Value {
            if cfg!(debug_assertions) {
                let has_returned = self.has_returned(builder);
                emit_debug_assert!(env, builder, has_returned);
            }
            return self.args().get_data(builder);
        }

        /// Stores the parent of this continuation, which may either be another
        /// continuation or the main stack. It is therefore represented as a
        /// `StackChain` element.
        pub fn set_parent_stack_chain<'a>(
            &mut self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            new_stack_chain: &StackChain,
        ) {
            let offset = wasmtime_continuations::offsets::continuation_object::PARENT_CHAIN;
            new_stack_chain.store(env, builder, self.address, offset)
        }
    }

    impl Payloads {
        pub(crate) fn new(base: ir::Value, offset: i32, pointer_type: ir::Type) -> Payloads {
            Payloads {
                base,
                offset,
                pointer_type,
            }
        }

        fn get(&self, builder: &mut FunctionBuilder, ty: ir::Type, offset: i32) -> ir::Value {
            let mem_flags = ir::MemFlags::trusted();
            builder
                .ins()
                .load(ty, mem_flags, self.base, self.offset + offset)
        }

        fn set<T>(&self, builder: &mut FunctionBuilder, offset: i32, value: ir::Value) {
            debug_assert_eq!(
                builder.func.dfg.value_type(value),
                Type::int_with_byte_size(std::mem::size_of::<T>() as u16).unwrap()
            );
            let mem_flags = ir::MemFlags::trusted();
            builder
                .ins()
                .store(mem_flags, value, self.base, self.offset + offset);
        }

        pub fn get_data(&self, builder: &mut FunctionBuilder) -> ir::Value {
            self.get(
                builder,
                self.pointer_type,
                wasmtime_continuations::offsets::payloads::DATA,
            )
        }

        fn get_capacity(&self, builder: &mut FunctionBuilder) -> ir::Value {
            let ty = Type::int_with_byte_size(std::mem::size_of::<
                wasmtime_continuations::types::payloads::Capacity,
            >() as u16)
            .unwrap();
            self.get(
                builder,
                ty,
                wasmtime_continuations::offsets::payloads::CAPACITY,
            )
        }

        fn get_length(&self, builder: &mut FunctionBuilder) -> ir::Value {
            let ty = Type::int_with_byte_size(std::mem::size_of::<
                wasmtime_continuations::types::payloads::Length,
            >() as u16)
            .unwrap();
            self.get(
                builder,
                ty,
                wasmtime_continuations::offsets::payloads::LENGTH,
            )
        }

        fn set_length(&self, builder: &mut FunctionBuilder, length: ir::Value) {
            self.set::<wasmtime_continuations::types::payloads::Length>(
                builder,
                wasmtime_continuations::offsets::payloads::LENGTH,
                length,
            );
        }

        fn set_capacity(&self, builder: &mut FunctionBuilder, capacity: ir::Value) {
            self.set::<wasmtime_continuations::types::payloads::Capacity>(
                builder,
                wasmtime_continuations::offsets::payloads::CAPACITY,
                capacity,
            );
        }

        fn set_data(&self, builder: &mut FunctionBuilder, data: ir::Value) {
            self.set::<*mut u8>(
                builder,
                wasmtime_continuations::offsets::payloads::DATA,
                data,
            );
        }

        /// Returns pointer to next empty slot in data buffer and marks the
        /// subsequent `arg_count` slots as occupied.
        pub fn occupy_next_slots<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            arg_count: i32,
        ) -> ir::Value {
            let data = self.get_data(builder);
            let original_length = self.get_length(builder);
            let new_length = builder.ins().iadd_imm(original_length, arg_count as i64);
            self.set_length(builder, new_length);

            if cfg!(debug_assertions) {
                let capacity = self.get_capacity(builder);
                emit_debug_assert_ule!(env, builder, new_length, capacity);
            }

            let value_size =
                mem::size_of::<wasmtime_continuations::types::payloads::DataEntries>() as i64;
            let byte_offset = builder.ins().imul_imm(original_length, value_size);
            builder.ins().iadd(data, byte_offset)
        }

        #[allow(dead_code)]
        pub fn deallocate_buffer<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
        ) {
            let zero = builder.ins().iconst(ir::types::I64, 0);
            let capacity = self.get_capacity(builder);
            emit_debug_assert_ne!(env, builder, capacity, zero);

            let align = builder.ins().iconst(
                I64,
                std::mem::align_of::<wasmtime_continuations::types::payloads::DataEntries>() as i64,
            );
            let entry_size =
                std::mem::size_of::<wasmtime_continuations::types::payloads::DataEntries>();
            let size = builder.ins().imul_imm(capacity, entry_size as i64);

            let index = BuiltinFunctionIndex::tc_deallocate();
            let sig = env.builtin_function_signatures.tc_deallocate(builder.func);

            let ptr = self.get_data(builder);
            env.generate_builtin_call_no_return_val(builder, index, sig, vec![ptr, size, align]);

            self.set_capacity(builder, zero);
            self.set_length(builder, zero);
            self.set_data(builder, zero);
        }

        pub fn ensure_capacity<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            required_capacity: ir::Value,
        ) {
            let zero = builder.ins().iconst(ir::types::I64, 0);
            emit_debug_assert_ne!(env, builder, required_capacity, zero);

            if cfg!(debug_assertions) {
                let data = self.get_data(builder);
                emit_debug_println!(
                    env,
                    builder,
                    "[ensure_capacity] contobj/base {:p}, buffer is {:p}",
                    self.base,
                    data
                );
            }

            let capacity = self.get_capacity(builder);

            let sufficient_capacity_block = builder.create_block();
            let insufficient_capacity_block = builder.create_block();

            let big_enough =
                builder
                    .ins()
                    .icmp(IntCC::UnsignedLessThanOrEqual, required_capacity, capacity);

            builder.ins().brif(
                big_enough,
                sufficient_capacity_block,
                &[],
                insufficient_capacity_block,
                &[],
            );

            {
                builder.switch_to_block(insufficient_capacity_block);
                builder.seal_block(insufficient_capacity_block);

                emit_debug_println!(
                    env,
                    builder,
                    "[ensure_capacity] need to increase capacity from {} to {}",
                    capacity,
                    required_capacity
                );

                if cfg!(debug_assertions) {
                    // We must only re-allocate while there is no data in the buffer.
                    let length = self.get_length(builder);
                    emit_debug_assert_eq!(env, builder, length, zero);
                }

                let align = builder.ins().iconst(
                    I64,
                    std::mem::align_of::<wasmtime_continuations::types::payloads::DataEntries>()
                        as i64,
                );
                let entry_size =
                    std::mem::size_of::<wasmtime_continuations::types::payloads::DataEntries>();
                let old_size = builder.ins().imul_imm(capacity, entry_size as i64);
                let new_size = builder.ins().imul_imm(required_capacity, entry_size as i64);

                let index = BuiltinFunctionIndex::tc_reallocate();
                let sig = env.builtin_function_signatures.tc_reallocate(builder.func);

                let ptr = self.get_data(builder);
                let (_, new_data) = env.generate_builtin_call(
                    builder,
                    index,
                    sig,
                    vec![ptr, old_size, new_size, align],
                );

                self.set_capacity(builder, required_capacity);
                self.set_data(builder, new_data);
                self.set_length(builder, zero);
                builder.ins().jump(sufficient_capacity_block, &[]);
            }

            builder.switch_to_block(sufficient_capacity_block);
            builder.seal_block(sufficient_capacity_block);
        }

        /// Loads n entries from this Payloads object, where n is the length of
        /// `load_types`, which also gives the types of the values to load.
        /// Loading starts at index 0 of the Payloads object.
        pub fn load_data_entries<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            load_types: &[ir::Type],
        ) -> Vec<ir::Value> {
            if cfg!(debug_assertions) {
                let length = self.get_length(builder);
                let load_count = builder.ins().iconst(I64, load_types.len() as i64);
                emit_debug_assert_ule!(env, builder, load_count, length);
            }

            let memflags = ir::MemFlags::trusted();

            let data_start_pointer = self.get_data(builder);
            let mut values = vec![];
            let mut offset = 0;
            for valtype in load_types {
                let val = builder
                    .ins()
                    .load(*valtype, memflags, data_start_pointer, offset);
                values.push(val);
                offset +=
                    std::mem::size_of::<wasmtime_continuations::types::payloads::DataEntries>()
                        as i32;
            }
            values
        }

        /// Stores the given `values` in this Payloads object, beginning at
        /// index 0. This expects the Payloads object to be empty (i.e., current
        /// length is 0), and to be of sufficient capacity to store |`values`|
        /// entries.
        pub fn store_data_entries<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            values: &[ir::Value],
        ) {
            let store_count = builder.ins().iconst(I64, values.len() as i64);

            if cfg!(debug_assertions) {
                let capacity = self.get_capacity(builder);
                let length = self.get_length(builder);
                let zero = builder.ins().iconst(I64, 0);
                emit_debug_assert_ule!(env, builder, store_count, capacity);
                emit_debug_assert_eq!(env, builder, length, zero);
            }

            let memflags = ir::MemFlags::trusted();

            let data_start_pointer = self.get_data(builder);

            let mut offset = 0;
            for value in values {
                builder
                    .ins()
                    .store(memflags, *value, data_start_pointer, offset);
                offset +=
                    std::mem::size_of::<wasmtime_continuations::types::payloads::DataEntries>()
                        as i32;
            }

            self.set_length(builder, store_count);
        }

        pub fn clear(&self, builder: &mut FunctionBuilder) {
            let zero = builder.ins().iconst(I64, 0);
            self.set_length(builder, zero);
        }

        /// Silences some unused function warnings
        #[allow(dead_code)]
        pub fn dummy<'a>(
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
        ) {
            let _index = BuiltinFunctionIndex::tc_allocate();
            let _sig = env.builtin_function_signatures.tc_allocate(builder.func);
            let _index = BuiltinFunctionIndex::tc_deallocate();
            let _sig = env.builtin_function_signatures.tc_deallocate(builder.func);
        }
    }

    impl VMContext {
        pub fn new(address: ir::Value, pointer_type: ir::Type) -> VMContext {
            VMContext {
                address,
                pointer_type,
            }
        }

        /// Returns the stack chain saved in this `VMContext`. Note that the
        /// head of the list is the actively running stack (main stack or
        /// continuation).
        pub fn load_stack_chain<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
        ) -> StackChain {
            let base_addr = self.address;

            let offset =
                i32::try_from(env.offsets.vmctx_typed_continuations_stack_chain()).unwrap();

            // The `typed_continuations_stack_chain` field of the VMContext only
            // contains a pointer to the `StackChainCell` in the `Store`.
            // The pointer never changes through the liftime of a `VMContext`,
            // which is why this load is `readonly`.
            // TODO(frank-emrich) Consider turning this pointer into a global
            // variable, similar to `env.vmruntime_limits_ptr`.
            let memflags = ir::MemFlags::trusted().with_readonly();
            let stack_chain_ptr =
                builder
                    .ins()
                    .load(self.pointer_type, memflags, base_addr, offset);

            StackChain::load(env, builder, stack_chain_ptr, 0, self.pointer_type)
        }

        /// Stores the given stack chain saved in this `VMContext`, overwriting
        /// the exsiting one.
        pub fn store_stack_chain<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            stack_chain: &StackChain,
        ) {
            let base_addr = self.address;

            let offset =
                i32::try_from(env.offsets.vmctx_typed_continuations_stack_chain()).unwrap();

            // Same situation as in `load_stack_chain` regarding pointer
            // indirection and it being `readonly`.
            let memflags = ir::MemFlags::trusted().with_readonly();
            let stack_chain_ptr =
                builder
                    .ins()
                    .load(self.pointer_type, memflags, base_addr, offset);

            stack_chain.store(env, builder, stack_chain_ptr, 0)
        }

        /// Similar to `store_stack_chain`, but instead of storing an arbitrary
        /// `StackChain`, stores StackChain::Continuation(contobj)`.
        pub fn set_active_continuation<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            contobj: ir::Value,
        ) {
            let chain = StackChain::from_continuation(builder, contobj, self.pointer_type);
            self.store_stack_chain(env, builder, &chain)
        }
    }

    impl StackChain {
        /// Creates a `Self` corressponding to `StackChain::Continuation(contobj)`.
        pub fn from_continuation(
            builder: &mut FunctionBuilder,
            contobj: ir::Value,
            pointer_type: ir::Type,
        ) -> StackChain {
            debug_assert_eq!(STACK_CHAIN_POINTER_COUNT, 2);
            let discriminant = wasmtime_continuations::StackChain::CONTINUATION_DISCRIMINANT;
            let discriminant = builder.ins().iconst(pointer_type, discriminant as i64);
            StackChain {
                discriminant,
                payload: contobj,
                pointer_type,
            }
        }

        /// Creates a `Self` corressponding to `StackChain::Absent`.
        pub fn absent(builder: &mut FunctionBuilder, pointer_type: ir::Type) -> StackChain {
            debug_assert_eq!(STACK_CHAIN_POINTER_COUNT, 2);
            let discriminant = wasmtime_continuations::StackChain::ABSENT_DISCRIMINANT;
            let discriminant = builder.ins().iconst(pointer_type, discriminant as i64);
            let zero_filler = builder.ins().iconst(pointer_type, 0i64);
            StackChain {
                discriminant,
                payload: zero_filler,
                pointer_type,
            }
        }

        /// For debugging purposes. Emits an assertion that `self` does not correspond to
        /// `StackChain::Absent`.
        pub fn assert_not_absent<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
        ) {
            let discriminant = wasmtime_continuations::StackChain::ABSENT_DISCRIMINANT;
            let discriminant = builder.ins().iconst(self.pointer_type, discriminant as i64);
            emit_debug_assert_ne!(env, builder, self.discriminant, discriminant);
        }

        /// Return the two raw `ir::Value`s that represent this StackChain.
        pub fn to_raw_parts(&self) -> [ir::Value; STACK_CHAIN_POINTER_COUNT] {
            [self.discriminant, self.payload]
        }

        /// Construct a `Self` from two raw `ir::Value`s.
        pub fn from_raw_parts(
            raw_data: [ir::Value; STACK_CHAIN_POINTER_COUNT],
            pointer_type: ir::Type,
        ) -> StackChain {
            StackChain {
                discriminant: raw_data[0],
                payload: raw_data[1],
                pointer_type,
            }
        }

        /// Load a `StackChain` object from the given address.
        pub fn load<'a>(
            _env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            pointer: ir::Value,
            initial_offset: i32,
            pointer_type: ir::Type,
        ) -> StackChain {
            let memflags = ir::MemFlags::trusted();
            let mut offset = initial_offset;
            let mut data = vec![];
            for _ in 0..STACK_CHAIN_POINTER_COUNT {
                data.push(builder.ins().load(pointer_type, memflags, pointer, offset));
                offset += pointer_type.bytes() as i32;
            }
            let data = <[ir::Value; STACK_CHAIN_POINTER_COUNT]>::try_from(data).unwrap();
            Self::from_raw_parts(data, pointer_type)
        }

        /// Store this `StackChain` object at the given address.
        pub fn store<'a>(
            &self,
            _env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            target_pointer: ir::Value,
            initial_offset: i32,
        ) {
            let memflags = ir::MemFlags::trusted();
            let mut offset = initial_offset;
            let data = self.to_raw_parts();

            for value in data {
                debug_assert_eq!(
                    builder.func.dfg.value_type(value),
                    Type::int_with_byte_size(self.pointer_type.bytes() as u16).unwrap()
                );
                builder.ins().store(memflags, value, target_pointer, offset);
                offset += self.pointer_type.bytes() as i32;
            }
        }

        /// If `self` corresponds to a `StackChain::Continuation`, return the
        /// pointer to the continuation object stored in the variant.
        /// If `self` corresponds to `StackChain::MainStack`, trap with the
        /// given `trap_code`.
        /// Calling this if `self` corresponds to `StackChain::Absent` indicates
        /// an internal bug.
        pub fn unwrap_continuation_or_trap<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            trap_code: ir::TrapCode,
        ) -> ir::Value {
            if cfg!(debug_assertions) {
                let absent_discriminant = wasmtime_continuations::StackChain::ABSENT_DISCRIMINANT;
                let is_initialized = builder.ins().icmp_imm(
                    IntCC::NotEqual,
                    self.discriminant,
                    absent_discriminant as i64,
                );
                emit_debug_assert!(env, builder, is_initialized);
            }

            let continuation_discriminant =
                wasmtime_continuations::StackChain::CONTINUATION_DISCRIMINANT;
            let is_continuation = builder.ins().icmp_imm(
                IntCC::Equal,
                self.discriminant,
                continuation_discriminant as i64,
            );
            builder.ins().trapz(is_continuation, trap_code);

            // The representation of StackChain::Continuation stores
            // the pointer right after the discriminant.
            self.payload
        }

        /// Must only be called if `self` represents a `MainStack` or `Continuation` variant.
        /// Returns a pointer to the associated `StackLimits` object (i.e., in
        /// the former case, the pointer directly stored in the variant, or in
        /// the latter case a pointer to the `StackLimits` data within the
        /// `ContinuationObject`.
        pub fn get_stack_limits_ptr<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
        ) -> ir::Value {
            use wasmtime_continuations::offsets as o;

            self.assert_not_absent(env, builder);

            // `self` corresponds to a StackChain::MainStack or
            // StackChain::Continuation.
            // In both cases, the payload is a pointer.
            let ptr = self.payload;

            // `obj` is now a pointer to the beginning of either
            // 1. A ContinuationObject object (in the case of a
            // StackChain::Continuation)
            // 2. A StackLimits object (in the case of
            // StackChain::MainStack)
            //
            // Since a ContinuationObject starts with an (inlined) StackLimits
            // object at offset 0, we actually have in both cases that `ptr` is
            // now the address of the beginning of a StackLimits object.
            debug_assert_eq!(o::continuation_object::LIMITS, 0);
            ptr
        }

        /// Sets `last_wasm_entry_sp` and `stack_limit` fields in
        /// `VMRuntimelimits` using the values from the `StackLimits` object
        /// associated with this stack chain.
        pub fn write_limits_to_vmcontext<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            vmruntime_limits: cranelift_frontend::Variable,
        ) {
            use wasmtime_continuations::offsets as o;

            let stack_limits_ptr = self.get_stack_limits_ptr(env, builder);
            let vmruntime_limits_ptr = builder.use_var(vmruntime_limits);

            let memflags = ir::MemFlags::trusted();

            let mut copy_to_vm_runtime_limits = |our_offset, their_offset| {
                let our_value =
                    builder
                        .ins()
                        .load(self.pointer_type, memflags, stack_limits_ptr, our_offset);
                builder
                    .ins()
                    .store(memflags, our_value, vmruntime_limits_ptr, their_offset);
            };

            let pointer_size = self.pointer_type.bytes() as u8;
            copy_to_vm_runtime_limits(
                o::stack_limits::STACK_LIMIT,
                pointer_size.vmruntime_limits_stack_limit(),
            );
            copy_to_vm_runtime_limits(
                o::stack_limits::LAST_WASM_ENTRY_SP,
                pointer_size.vmruntime_limits_last_wasm_entry_sp(),
            );
        }

        /// Overwrites the `last_wasm_entry_sp` field of the `StackLimits`
        /// object associated with this stack chain by loading the corresponding
        /// field from the `VMRuntimeLimits`.
        pub fn load_limits_from_vmcontext<'a>(
            &self,
            env: &mut crate::func_environ::FuncEnvironment<'a>,
            builder: &mut FunctionBuilder,
            vmruntime_limits: cranelift_frontend::Variable,
        ) {
            use wasmtime_continuations::offsets as o;

            let stack_limits_ptr = self.get_stack_limits_ptr(env, builder);
            let vmruntime_limits_ptr = builder.use_var(vmruntime_limits);

            let memflags = ir::MemFlags::trusted();
            let pointer_size = self.pointer_type.bytes() as u8;

            let last_wasm_entry_sp = builder.ins().load(
                self.pointer_type,
                memflags,
                vmruntime_limits_ptr,
                pointer_size.vmruntime_limits_last_wasm_entry_sp(),
            );
            builder.ins().store(
                memflags,
                last_wasm_entry_sp,
                stack_limits_ptr,
                o::stack_limits::LAST_WASM_ENTRY_SP,
            );
        }
    }
}

use typed_continuation_helpers as tc;

fn typed_continuations_load_return_values<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    valtypes: &[WasmValType],
    contobj: ir::Value,
) -> std::vec::Vec<ir::Value> {
    let co = tc::ContinuationObject::new(contobj, env.pointer_type());
    let mut values = vec![];

    if valtypes.len() > 0 {
        let result_buffer_addr = co.get_results(env, builder);

        let mut offset = 0;
        let memflags = ir::MemFlags::trusted();
        for valtype in valtypes {
            let val = builder.ins().load(
                crate::value_type(env.isa, *valtype),
                memflags,
                result_buffer_addr,
                offset,
            );
            values.push(val);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }
    }
    return values;
}

fn typed_continuations_forward_tag_return_values<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    parent_contobj: ir::Value,
    child_contobj: ir::Value,
) {
    shared::generate_builtin_call_no_return_val!(
        env,
        builder,
        tc_cont_obj_forward_tag_return_values_buffer,
        [parent_contobj, child_contobj]
    );
}

fn typed_continuations_load_payloads<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    valtypes: &[ir::Type],
) -> Vec<ir::Value> {
    let mut values = vec![];

    if valtypes.len() > 0 {
        let vmctx = env.vmctx(builder.cursor().func);
        let vmctx = builder.ins().global_value(env.pointer_type(), vmctx);
        let vmctx_payloads = tc::Payloads::new(
            vmctx,
            env.offsets.vmctx_typed_continuations_payloads() as i32,
            env.pointer_type(),
        );

        values = vmctx_payloads.load_data_entries(env, builder, valtypes);

        // In theory, we way want to deallocate the buffer instead of just
        // clearing it if its size is above a certain threshold. That would
        // avoid keeping a large object unnecessarily long.
        vmctx_payloads.clear(builder);
    }

    values
}

pub(crate) fn typed_continuations_load_tag_return_values<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
    valtypes: &[WasmValType],
) -> Vec<ir::Value> {
    let memflags = ir::MemFlags::trusted();
    let mut values = vec![];

    if valtypes.len() > 0 {
        let co = tc::ContinuationObject::new(contobj, env.pointer_type());
        let tag_return_values = co.tag_return_values();

        let payload_ptr = tag_return_values.get_data(builder);

        let mut offset = 0;
        for valtype in valtypes {
            let val = builder.ins().load(
                crate::value_type(env.isa, *valtype),
                memflags,
                payload_ptr,
                offset,
            );
            values.push(val);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }

        // In theory, we way want to deallocate the buffer instead of just
        // clearing it if its size is above a certain threshold. That would
        // avoid keeping a large object unnecessarily long.
        tag_return_values.clear(builder);
    }

    values
}

/// TODO
pub(crate) fn typed_continuations_store_resume_args<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    values: &[ir::Value],
    remaining_arg_count: ir::Value,
    contobj: ir::Value,
) {
    if values.len() > 0 {
        let use_args_block = builder.create_block();
        let use_payloads_block = builder.create_block();
        let store_data_block = builder.create_block();
        builder.append_block_param(store_data_block, env.pointer_type());

        let co = tc::ContinuationObject::new(contobj, env.pointer_type());
        let is_invoked = co.is_invoked(builder);
        builder
            .ins()
            .brif(is_invoked, use_payloads_block, &[], use_args_block, &[]);

        {
            builder.switch_to_block(use_args_block);
            builder.seal_block(use_args_block);

            let args = co.args();
            let ptr = args.occupy_next_slots(env, builder, values.len() as i32);

            builder.ins().jump(store_data_block, &[ptr]);
        }

        {
            builder.switch_to_block(use_payloads_block);
            builder.seal_block(use_payloads_block);

            let tag_return_values = co.tag_return_values();

            // Unlike for the args buffer (where we know the maximum
            // required capacity at the time of creation of the
            // ContinuationObject), tag return buffers are re-used and may
            // be too small.
            tag_return_values.ensure_capacity(env, builder, remaining_arg_count);

            let ptr = tag_return_values.occupy_next_slots(env, builder, values.len() as i32);
            builder.ins().jump(store_data_block, &[ptr]);
        }

        {
            builder.switch_to_block(store_data_block);
            builder.seal_block(store_data_block);

            let ptr = builder.block_params(store_data_block)[0];

            // Store the values.
            let memflags = ir::MemFlags::trusted();
            let mut offset = 0;
            for value in values {
                builder.ins().store(memflags, *value, ptr, offset);
                offset += env.offsets.ptr.maximum_value_size() as i32;
            }
        }
    }
}

//TODO(frank-emrich) Consider removing `valtypes` argument, as values are inherently typed
pub(crate) fn typed_continuations_store_payloads<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    valtypes: &[WasmValType],
    values: &[ir::Value],
) {
    assert_eq!(values.len(), valtypes.len());
    if valtypes.len() > 0 {
        let vmctx = env.vmctx(builder.cursor().func);
        let vmctx = builder.ins().global_value(env.pointer_type(), vmctx);
        let payloads = tc::Payloads::new(
            vmctx,
            env.offsets.vmctx_typed_continuations_payloads() as i32,
            env.pointer_type(),
        );

        let nargs = builder.ins().iconst(I64, values.len() as i64);
        payloads.ensure_capacity(env, builder, nargs);

        payloads.store_data_entries(env, builder, values);
    }
}

pub(crate) fn typed_continuations_load_continuation_object<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
) -> ir::Value {
    let vmctx = env.vmctx(builder.cursor().func);
    let vmctx = builder.ins().global_value(env.pointer_type(), vmctx);
    let vmctx = tc::VMContext::new(vmctx, env.pointer_type());
    let active_stack_chain = vmctx.load_stack_chain(env, builder);
    active_stack_chain.unwrap_continuation_or_trap(
        env,
        builder,
        ir::TrapCode::User(crate::DEBUG_ASSERT_TRAP_CODE),
    )
}

pub(crate) fn translate_cont_new<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    _state: &FuncTranslationState,
    func: ir::Value,
    arg_types: &[WasmValType],
    return_types: &[WasmValType],
) -> WasmResult<ir::Value> {
    let nargs = builder.ins().iconst(I64, arg_types.len() as i64);
    let nreturns = builder.ins().iconst(I64, return_types.len() as i64);

    let (_vmctx, contobj) =
        shared::generate_builtin_call!(env, builder, tc_cont_new, [func, nargs, nreturns]);

    Ok(contobj)
}

pub(crate) fn translate_resume<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    type_index: u32,
    contref: ir::Value,
    resume_args: &[ir::Value],
    resumetable: &[(u32, ir::Block)],
) -> Vec<ir::Value> {
    let resume_block = builder.create_block();
    let return_block = builder.create_block();
    let suspend_block = builder.create_block();
    let switch_block = builder.create_block();
    let forwarding_block = builder.create_block();

    let vmctx = env.vmctx(builder.func);
    let vmctx = builder.ins().global_value(env.pointer_type(), vmctx);

    // Preamble: Part of previously active block

    let (resume_contobj, parent_stack_chain) = {
        let resume_contobj =
            shared::typed_continuations_cont_ref_get_cont_obj(env, builder, contref);

        if resume_args.len() > 0 {
            // We store the arguments in the continuation object to be resumed.
            let count = builder.ins().iconst(I64, resume_args.len() as i64);
            typed_continuations_store_resume_args(env, builder, resume_args, count, resume_contobj);
        }

        // Make the currently running continuation (if any) the parent of the one we are about to resume.
        let original_stack_chain =
            tc::VMContext::new(vmctx, env.pointer_type()).load_stack_chain(env, builder);
        original_stack_chain.assert_not_absent(env, builder);
        tc::ContinuationObject::new(resume_contobj, env.pointer_type()).set_parent_stack_chain(
            env,
            builder,
            &original_stack_chain,
        );

        builder.ins().jump(resume_block, &[]);
        (resume_contobj, original_stack_chain)
    };

    // Resume block: actually resume the fiber corresponding to the
    // continuation object given as a parameter to the block. This
    // parameterisation is necessary to enable forwarding, requiring us
    // to resume objects other than `original_contobj`.
    // We make the continuation object that was actually resumed available via
    // `resumed_contobj`, so that subsequent blocks can refer to it.
    let resume_result = {
        builder.switch_to_block(resume_block);

        // We mark `resume_contobj` as the currently running one
        let vmctx = tc::VMContext::new(vmctx, env.pointer_type());
        vmctx.set_active_continuation(env, builder, resume_contobj);

        // Note that the resume_contobj libcall a few lines further below
        // manipulates the stack limits as follows:
        // 1. Copy stack_limit, last_wasm_entry_sp and last_wasm_exit* values from
        // VMRuntimeLimits into the currently active continuation (i.e., the
        // one that will become the parent of the to-be-resumed one)
        //
        // 2. Copy `stack_limit` and `last_wasm_entry_sp` in the
        // `StackLimits` of `resume_contobj` into the `VMRuntimeLimits`.
        //
        // See the comment on `wasmtime_continuations::StackChain` for a
        // description of the invariants that we maintain for the various stack
        // limits.
        let parent_stacks_limit_pointer = parent_stack_chain.get_stack_limits_ptr(env, builder);

        // We mark `resume_contobj` to be invoked
        let co = tc::ContinuationObject::new(resume_contobj, env.pointer_type());
        co.set_state(builder, wasmtime_continuations::State::Invoked);

        let (_vmctx, result) = shared::generate_builtin_call!(
            env,
            builder,
            tc_resume,
            [resume_contobj, parent_stacks_limit_pointer]
        );

        emit_debug_println!(
            env,
            builder,
            "[resume] libcall finished, result is {:p}",
            result
        );

        // Now the parent contobj (or main stack) is active again
        vmctx.store_stack_chain(env, builder, &parent_stack_chain);

        // The `result` is a value of type wasmtime_fibre::SwitchDirection,
        // using the encoding described at its definition.
        // Thus, the first 32 bit encode the discriminant, and the
        // subsequent 32 bit encode the tag if suspending, or 0 otherwise.
        // Thus, when returning, the overall u64 should be zero.
        let return_discriminant =
            wasmtime_continuations::SwitchDirectionEnum::Return.discriminant_val();
        debug_assert_eq!(return_discriminant, 0);

        // If these two assumptions don't hold anymore, the code here becomes invalid.
        debug_assert_eq!(
            std::mem::size_of::<wasmtime_continuations::types::switch_reason::Discriminant>(),
            4
        );
        debug_assert_eq!(
            std::mem::size_of::<wasmtime_continuations::types::switch_reason::Data>(),
            4
        );

        if cfg!(debug_assertions) {
            let discriminant = builder.ins().ireduce(I32, result);
            emit_debug_println!(env, builder, "[resume] discriminant is {}", discriminant);
        }

        // Jump to the return block if the result is 0, otherwise jump to
        // the suspend block.
        builder
            .ins()
            .brif(result, suspend_block, &[], return_block, &[]);

        // We do not seal this block, yet, because the effect forwarding block has a back edge to it
        result
    };

    // Suspend block.
    let tag = {
        builder.switch_to_block(suspend_block);
        builder.seal_block(suspend_block);

        // We store parts of the VMRuntimeLimits into the continuation that just suspended.
        let suspended_chain =
            tc::StackChain::from_continuation(builder, resume_contobj, env.pointer_type());
        suspended_chain.load_limits_from_vmcontext(env, builder, env.vmruntime_limits_ptr);

        // Afterwards (!), restore parts of the VMRuntimeLimits from the
        // parent of the suspended continuation (which is now active).
        parent_stack_chain.write_limits_to_vmcontext(env, builder, env.vmruntime_limits_ptr);

        let discriminant = builder.ins().ireduce(I32, resume_result);
        let discriminant_size_bytes =
            std::mem::size_of::<wasmtime_continuations::types::switch_reason::Discriminant>();

        if cfg!(debug_assertions) {
            let suspend_discriminant =
                wasmtime_continuations::SwitchDirectionEnum::Suspend.discriminant_val();
            let suspend_discriminant = builder.ins().iconst(I32, suspend_discriminant as i64);
            emit_debug_assert_eq!(env, builder, discriminant, suspend_discriminant);
        }

        let tag = builder
            .ins()
            .ushr_imm(resume_result, discriminant_size_bytes as i64 * 8);
        let tag = builder.ins().ireduce(I32, tag);

        emit_debug_println!(env, builder, "[resume] in suspend block, tag is {}", tag);

        // We need to terminate this block before being allowed to switch to another one
        builder.ins().jump(switch_block, &[]);

        tag
    };

    // Now, construct blocks for the three continuations:
    // 1) `resume` returned normally.
    // 2) `resume` returned via a suspend.
    // 3) `resume` is forwarding

    // Strategy:
    //
    // Translate each each `(tag, label)` pair in the resume table
    // to a switch-case of the form "case tag: br label". NOTE:
    // `tag` may appear multiple times in resume table, only the
    // first appearance should be processed as it shadows the
    // subsequent entries.  The switching logic then ensures that
    // we jump to the block handling the corresponding tag.
    //
    // The fallback/default case performs effect forwarding (TODO).
    //
    // First, initialise the switch structure.
    let mut switch = Switch::new();
    // Second, we consume the resume table entry-wise.
    let mut case_blocks = vec![];
    let mut tag_seen = std::collections::HashSet::new(); // Used to keep track of tags
    for &(tag, target_block) in resumetable {
        // Skip if this `tag` has been seen previously.
        if !tag_seen.insert(tag) {
            continue;
        }
        let case = builder.create_block();
        switch.set_entry(tag as u128, case);
        builder.switch_to_block(case);

        // Load and push arguments.
        let param_types = env.tag_params(tag);
        let param_types: Vec<ir::Type> = param_types
            .iter()
            .map(|wty| crate::value_type(env.isa, *wty))
            .collect();
        let mut args = typed_continuations_load_payloads(env, builder, &param_types);

        // We have an actual handling block for this tag, rather than just
        // forwarding. Detatch the continuation object by setting its parent
        // link to `StackChain::Absent`.
        let pointer_type = env.pointer_type();
        let chain = tc::StackChain::absent(builder, pointer_type);
        tc::ContinuationObject::new(resume_contobj, pointer_type)
            .set_parent_stack_chain(env, builder, &chain);

        // Create and push the continuation reference. We only create
        // them here because we don't need them when forwarding.
        let contref = env.typed_continuations_new_cont_ref(builder, resume_contobj);

        args.push(contref);

        // Now jump to the actual user-defined block handling
        // this tag, as given by the resumetable.
        builder.ins().jump(target_block, &args);
        case_blocks.push(case);
    }

    // Note that at this point we haven't actually emitted any
    // code for the switching logic itself, but only filled
    // the Switch structure and created the blocks it jumps
    // to.

    // Forwarding block: Default case for the switching logic on the
    // tag. Used when the (resume ...) clause we currently translate
    // does not have a matching (tag ...) entry.
    {
        builder.switch_to_block(forwarding_block);

        let parent_contobj = parent_stack_chain.unwrap_continuation_or_trap(
            env,
            builder,
            ir::TrapCode::UnhandledTag,
        );

        // We suspend, thus deferring handling to the parent.
        // We do nothing about tag *parameters*, these remain unchanged within the
        // payload buffer associated with the whole VMContext.
        shared::generate_builtin_call_no_return_val!(env, builder, tc_suspend, [tag]);

        // "Tag return values" (i.e., values provided by cont.bind or
        // resume to the continuation) are actually stored in
        // continuation objects, and we need to move them down the chain
        // back to the continuation object where we originally
        // suspended.
        typed_continuations_forward_tag_return_values(env, builder, parent_contobj, resume_contobj);

        // We create a back edge to the resume block.
        // Note that both `resume_cotobj` and `parent_stack_chain` remain unchanged:
        // In the current design, where forwarding is implemented by suspending
        // up the chain of parent continuations and subsequently resume-ing back
        // down the chain, both the continuation being resumed and its parent
        // stay the same.
        builder.ins().jump(resume_block, &[]);
        builder.seal_block(resume_block);
    }

    // Switch block: actual switching logic is emitted here.
    {
        builder.switch_to_block(switch_block);
        switch.emit(builder, tag, forwarding_block);
        builder.seal_block(switch_block);
        builder.seal_block(forwarding_block);

        // We can only seal the blocks we generated for each
        // tag now, after switch.emit ran.
        for case_block in case_blocks {
            builder.seal_block(case_block);
        }
    }

    // Return block: Jumped to by resume block if continuation returned normally.
    {
        builder.switch_to_block(return_block);
        builder.seal_block(return_block);

        // Restore parts of the VMRuntimeLimits from the
        // parent of the returned continuation (which is now active).
        parent_stack_chain.write_limits_to_vmcontext(env, builder, env.vmruntime_limits_ptr);

        let co = tc::ContinuationObject::new(resume_contobj, env.pointer_type());
        co.set_state(builder, wasmtime_continuations::State::Returned);

        // Load and push the results.
        let returns = env.continuation_returns(type_index).to_vec();
        let values = typed_continuations_load_return_values(env, builder, &returns, resume_contobj);

        // The continuation has returned and all `ContinuationReferences`
        // to it should have been be invalidated. We may safely deallocate
        // it.
        shared::typed_continuations_drop_cont_obj(env, builder, resume_contobj);

        return values;
    }
}

pub(crate) fn translate_suspend<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    tag_index: ir::Value,
) -> ir::Value {
    // Returns the vmctx
    return shared::generate_builtin_call_no_return_val!(env, builder, tc_suspend, [tag_index]);
}
