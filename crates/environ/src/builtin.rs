/// Helper macro to iterate over all builtin functions and their signatures.
#[macro_export]
macro_rules! foreach_builtin_function {
    ($mac:ident) => {
        $mac! {
            /// Returns an index for wasm's `memory.grow` builtin function.
            memory32_grow(vmctx: vmctx, delta: i64, index: i32) -> pointer;
            /// Returns an index for wasm's `table.copy` when both tables are locally
            /// defined.
            table_copy(vmctx: vmctx, dst_index: i32, src_index: i32, dst: i32, src: i32, len: i32);
            /// Returns an index for wasm's `table.init`.
            table_init(vmctx: vmctx, table: i32, elem: i32, dst: i32, src: i32, len: i32);
            /// Returns an index for wasm's `elem.drop`.
            elem_drop(vmctx: vmctx, elem: i32);
            /// Returns an index for wasm's `memory.copy`
            memory_copy(vmctx: vmctx, dst_index: i32, dst: i64, src_index: i32, src: i64, len: i64);
            /// Returns an index for wasm's `memory.fill` instruction.
            memory_fill(vmctx: vmctx, memory: i32, dst: i64, val: i32, len: i64);
            /// Returns an index for wasm's `memory.init` instruction.
            memory_init(vmctx: vmctx, memory: i32, data: i32, dst: i64, src: i32, len: i32);
            /// Returns a value for wasm's `ref.func` instruction.
            ref_func(vmctx: vmctx, func: i32) -> pointer;
            /// Returns an index for wasm's `data.drop` instruction.
            data_drop(vmctx: vmctx, data: i32);
            /// Returns a table entry after lazily initializing it.
            table_get_lazy_init_func_ref(vmctx: vmctx, table: i32, index: i32) -> pointer;
            /// Returns an index for Wasm's `table.grow` instruction for `funcref`s.
            table_grow_func_ref(vmctx: vmctx, table: i32, delta: i32, init: pointer) -> i32;
            /// Returns an index for Wasm's `table.fill` instruction for `funcref`s.
            table_fill_func_ref(vmctx: vmctx, table: i32, dst: i32, val: pointer, len: i32);
            /// Returns an index for wasm's `memory.atomic.notify` instruction.
            memory_atomic_notify(vmctx: vmctx, memory: i32, addr: i64, count: i32) -> i32;
            /// Returns an index for wasm's `memory.atomic.wait32` instruction.
            memory_atomic_wait32(vmctx: vmctx, memory: i32, addr: i64, expected: i32, timeout: i64) -> i32;
            /// Returns an index for wasm's `memory.atomic.wait64` instruction.
            memory_atomic_wait64(vmctx: vmctx, memory: i32, addr: i64, expected: i64, timeout: i64) -> i32;
            /// Invoked when fuel has run out while executing a function.
            out_of_gas(vmctx: vmctx);
            /// Invoked when we reach a new epoch.
            new_epoch(vmctx: vmctx) -> i64;

            /// Creates a new continuation from a funcref.
            tc_cont_new(vmctx: vmctx, r: pointer, param_count: i64, result_count: i64) -> pointer;
            /// Resumes a continuation. The result value is of type wasmtime_fibre::SwitchDirection.
            tc_resume(vmctx: vmctx, contobj: pointer, parent_stack_limits: pointer) -> i64;
            /// Suspends a continuation.
            tc_suspend(vmctx: vmctx, tag: i32);
            /// Returns the continuation object corresponding to the given continuation reference.
            tc_cont_ref_get_cont_obj(vmctx: vmctx, contref: pointer) -> pointer;
            /// Drops the given continuation object. Currently unused.
            //cont_obj_drop(vmctx: vmctx, contobj: pointer);
            /// Crates a new continuation reference.
            tc_new_cont_ref(vmctx: vmctx, contobj: pointer) -> pointer;


            /// Sets the tag return values of `child_contobj` to those of `parent_contobj`.
            /// This is implemented by exchanging the pointers to the underlying buffers.
            /// `child_contobj` must not currently have a tag return value buffer.
            /// `parent_contobj` may or may not have one.
            tc_cont_obj_forward_tag_return_values_buffer(vmctx: vmctx, parent_contobj: pointer, child_contobj : pointer);

            /// TODO
            tc_drop_cont_obj(vmctx: vmctx, contobj: pointer);

            /// General-purpose allocation. Only used by typed-continuations
            /// code at the moment.
            tc_allocate(vmctx: vmctx, size: i64, align: i64) -> pointer;
            /// General-purpose deallocation. Only used by typed-continuations
            /// code at the moment.
            tc_deallocate(vmctx: vmctx, ptr: pointer, size: i64, align: i64);
            /// General-purpose reallocation without preserving existing data. Concretely, behaves like
            /// deallocate followed by allocate.
            /// The only difference is that if `old_size` is 0, then we assume that ptr does not point to allocated memory
            /// and do not actually deallocate.
            /// `old_size` must be smaller than `new_size`
            tc_reallocate(vmctx: vmctx, ptr: pointer, old_size: i64, new_size: i64, align: i64) -> pointer;

            // TC baseline
            /// Baseline resume
            tc_baseline_resume(vmctx: vmctx, contobj: pointer) -> i32;
            /// Baseline suspend
            tc_baseline_suspend(vmctx: vmctx, tag: i32);
            /// Like suspend, but forwards handling.
            tc_baseline_forward(vmctx: vmctx, tag: i32, subcont: pointer);
            /// Baseline cont.new
            tc_baseline_cont_new(vmctx: vmctx, r: pointer, param_count: i64, result_count: i64) -> pointer;
            /// Baseline continuation drop
            tc_baseline_drop_continuation_reference(vmctx: vmctx, r: pointer);
            /// Baseline continuation arguments pointer
            tc_baseline_continuation_arguments_ptr(vmctx: vmctx, r: pointer, nargs: i64) -> pointer;
            /// Baseline continuation values pointer
            tc_baseline_continuation_values_ptr(vmctx: vmctx, r: pointer) -> pointer;
            /// Baseline continuation clear arguments
            tc_baseline_clear_arguments(vmctx: vmctx, r: pointer);
            /// Baseline suspend buffer pointer.
            tc_baseline_get_payloads_ptr(vmctx: vmctx, nargs: i64) -> pointer;
            /// Baseline clear suspend buffer.
            tc_baseline_clear_payloads(vmctx: vmctx);
            /// Baseline get current continuation.
            tc_baseline_get_current_continuation(vmctx: vmctx) -> pointer;

            // General-purpose printing functions.
            //
            /// Prints a string. Note that we transfer the string not as C strings, but as 'static str,
            /// represented as a pointer and a length.
            tc_print_str(vmctx: vmctx, s: pointer, len : i64);
            /// TODO
            tc_print_int(vmctx: vmctx, arg : i64);
            /// TODO
            tc_print_pointer(vmctx: vmctx, arg : pointer);

            /// Invoked before malloc returns.
            check_malloc(vmctx: vmctx, addr: i32, len: i32) -> i32;
            /// Invoked before the free returns.
            check_free(vmctx: vmctx, addr: i32) -> i32;
            /// Invoked before a load is executed.
            check_load(vmctx: vmctx, num_bytes: i32, addr: i32, offset: i32) -> i32;
            /// Invoked before a store is executed.
            check_store(vmctx: vmctx, num_bytes: i32, addr: i32, offset: i32) -> i32;
            /// Invoked after malloc is called.
            malloc_start(vmctx: vmctx);
            /// Invoked after free is called.
            free_start(vmctx: vmctx);
            /// Invoked when wasm stack pointer is updated.
            update_stack_pointer(vmctx: vmctx, value: i32);
            /// Invoked before memory.grow is called.
            update_mem_size(vmctx: vmctx, num_bytes: i32);

            /// Returns an index to drop a `VMExternRef`.
            #[cfg(feature = "gc")]
            drop_externref(vmctx: vmctx, val: pointer);

            /// Returns an index to do a GC and then insert a `VMExternRef` into the
            /// `VMExternRefActivationsTable`.
            #[cfg(feature = "gc")]
            activations_table_insert_with_gc(vmctx: vmctx, val: reference);

            /// Returns an index for Wasm's `global.get` instruction for `externref`s.
            #[cfg(feature = "gc")]
            externref_global_get(vmctx: vmctx, global: i32) -> reference;

            /// Returns an index for Wasm's `global.get` instruction for `externref`s.
            #[cfg(feature = "gc")]
            externref_global_set(vmctx: vmctx, global: i32, val: reference);

            /// Returns an index for Wasm's `table.grow` instruction for `externref`s.
            #[cfg(feature = "gc")]
            table_grow_externref(vmctx: vmctx, table: i32, delta: i32, init: reference) -> i32;

            /// Returns an index for Wasm's `table.fill` instruction for `externref`s.
            #[cfg(feature = "gc")]
            table_fill_externref(vmctx: vmctx, table: i32, dst: i32, val: reference, len: i32);
        }
    };
}

/// An index type for builtin functions.
#[derive(Copy, Clone, Debug)]
pub struct BuiltinFunctionIndex(u32);

impl BuiltinFunctionIndex {
    /// Create a new `BuiltinFunctionIndex` from its index
    pub const fn from_u32(i: u32) -> Self {
        Self(i)
    }

    /// Return the index as an u32 number.
    pub const fn index(&self) -> u32 {
        self.0
    }
}

macro_rules! declare_indexes {
    (
        $(
            $( #[$attr:meta] )*
            $name:ident( $( $pname:ident: $param:ident ),* ) $( -> $result:ident )?;
        )*
    ) => {
        impl BuiltinFunctionIndex {
            declare_indexes!(
                @indices;
                0;
                $( $( #[$attr] )* $name; )*
            );
        }
    };

    // Base case: no more indices to declare, so define the total number of
    // function indices.
    (
        @indices;
        $len:expr;
    ) => {
        /// Returns the total number of builtin functions.
        pub const fn builtin_functions_total_number() -> u32 {
            $len
        }
    };

    // Recursive case: declare the next index, and then keep declaring the rest of
    // the indices.
    (
         @indices;
         $index:expr;
         $( #[$this_attr:meta] )*
         $this_name:ident;
         $(
             $( #[$rest_attr:meta] )*
             $rest_name:ident;
         )*
    ) => {
        $( #[$this_attr] )*
        pub const fn $this_name() -> Self {
            Self($index)
        }

        declare_indexes!(
            @indices;
            ($index + 1);
            $( $( #[$rest_attr] )* $rest_name; )*
        );
    }
}

foreach_builtin_function!(declare_indexes);
