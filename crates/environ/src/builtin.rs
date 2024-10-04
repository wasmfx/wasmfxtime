/// Helper macro to iterate over all builtin functions and their signatures.
#[macro_export]
macro_rules! foreach_builtin_function {
    ($mac:ident) => {
        $mac! {
            // Returns an index for wasm's `memory.grow` builtin function.
            memory32_grow(vmctx: vmctx, delta: i64, index: i32) -> pointer;
            // Returns an index for wasm's `table.copy` when both tables are locally
            // defined.
            table_copy(vmctx: vmctx, dst_index: i32, src_index: i32, dst: i64, src: i64, len: i64);
            // Returns an index for wasm's `table.init`.
            table_init(vmctx: vmctx, table: i32, elem: i32, dst: i64, src: i64, len: i64);
            // Returns an index for wasm's `elem.drop`.
            elem_drop(vmctx: vmctx, elem: i32);
            // Returns an index for wasm's `memory.copy`
            memory_copy(vmctx: vmctx, dst_index: i32, dst: i64, src_index: i32, src: i64, len: i64);
            // Returns an index for wasm's `memory.fill` instruction.
            memory_fill(vmctx: vmctx, memory: i32, dst: i64, val: i32, len: i64);
            // Returns an index for wasm's `memory.init` instruction.
            memory_init(vmctx: vmctx, memory: i32, data: i32, dst: i64, src: i32, len: i32);
            // Returns a value for wasm's `ref.func` instruction.
            ref_func(vmctx: vmctx, func: i32) -> pointer;
            // Returns an index for wasm's `data.drop` instruction.
            data_drop(vmctx: vmctx, data: i32);
            // Returns a table entry after lazily initializing it.
            table_get_lazy_init_func_ref(vmctx: vmctx, table: i32, index: i64) -> pointer;
            // Returns an index for Wasm's `table.grow` instruction for `funcref`s.
            table_grow_func_ref(vmctx: vmctx, table: i32, delta: i64, init: pointer) -> pointer;
            // Returns an index for Wasm's `table.fill` instruction for `funcref`s.
            table_fill_func_ref(vmctx: vmctx, table: i32, dst: i64, val: pointer, len: i64);
            // Returns an index for wasm's `memory.atomic.notify` instruction.
            #[cfg(feature = "threads")]
            memory_atomic_notify(vmctx: vmctx, memory: i32, addr: i64, count: i32) -> i32;
            // Returns an index for wasm's `memory.atomic.wait32` instruction.
            #[cfg(feature = "threads")]
            memory_atomic_wait32(vmctx: vmctx, memory: i32, addr: i64, expected: i32, timeout: i64) -> i32;
            // Returns an index for wasm's `memory.atomic.wait64` instruction.
            #[cfg(feature = "threads")]
            memory_atomic_wait64(vmctx: vmctx, memory: i32, addr: i64, expected: i64, timeout: i64) -> i32;
            // Invoked when fuel has run out while executing a function.
            out_of_gas(vmctx: vmctx);
            // Invoked when we reach a new epoch.
            new_epoch(vmctx: vmctx) -> i64;
            // Invoked before malloc returns.
            #[cfg(feature = "wmemcheck")]
            check_malloc(vmctx: vmctx, addr: i32, len: i32) -> i32;
            // Invoked before the free returns.
            #[cfg(feature = "wmemcheck")]
            check_free(vmctx: vmctx, addr: i32) -> i32;
            // Invoked before a load is executed.
            #[cfg(feature = "wmemcheck")]
            check_load(vmctx: vmctx, num_bytes: i32, addr: i32, offset: i32) -> i32;
            // Invoked before a store is executed.
            #[cfg(feature = "wmemcheck")]
            check_store(vmctx: vmctx, num_bytes: i32, addr: i32, offset: i32) -> i32;
            // Invoked after malloc is called.
            #[cfg(feature = "wmemcheck")]
            malloc_start(vmctx: vmctx);
            // Invoked after free is called.
            #[cfg(feature = "wmemcheck")]
            free_start(vmctx: vmctx);
            // Invoked when wasm stack pointer is updated.
            #[cfg(feature = "wmemcheck")]
            update_stack_pointer(vmctx: vmctx, value: i32);
            // Invoked before memory.grow is called.
            #[cfg(feature = "wmemcheck")]
            update_mem_size(vmctx: vmctx, num_bytes: i32);

            // Drop a non-stack GC reference (eg an overwritten table entry)
            // once it will no longer be used again. (Note: `val` is not of type
            // `reference` because it needn't appear in any stack maps, as it
            // must not be live after this call.)
            #[cfg(feature = "gc")]
            drop_gc_ref(vmctx: vmctx, val: i32);

            // Do a GC, treating the optional `root` as a GC root and returning
            // the updated `root` (so that, in the case of moving collectors,
            // callers have a valid version of `root` again).
            #[cfg(feature = "gc")]
            gc(vmctx: vmctx, root: reference) -> reference;

            // Allocate a new, uninitialized GC object and return a reference to
            // it.
            #[cfg(feature = "gc")]
            gc_alloc_raw(
                vmctx: vmctx,
                kind: i32,
                module_interned_type_index: i32,
                size: i32,
                align: i32
            ) -> reference;

            // Intern a `funcref` into the GC heap, returning its
            // `FuncRefTableId`.
            //
            // This libcall may not GC.
            #[cfg(feature = "gc")]
            intern_func_ref_for_gc_heap(
                vmctx: vmctx,
                func_ref: pointer
            ) -> i32;

            // Get the raw `VMFuncRef` pointer associated with a
            // `FuncRefTableId` from an earlier `intern_func_ref_for_gc_heap`
            // call.
            //
            // This libcall may not GC.
            //
            // Passes in the `ModuleInternedTypeIndex` of the funcref's expected
            // type, or `ModuleInternedTypeIndex::reserved_value()` if we are
            // getting the function reference as an untyped `funcref` rather
            // than a typed `(ref $ty)`.
            //
            // TODO: We will want to eventually expose the table directly to
            // Wasm code, so that it doesn't need to make a libcall to go from
            // id to `VMFuncRef`. That will be a little tricky: it will also
            // require updating the pointer to the slab in the `VMContext` (or
            // `VMRuntimeLimits` or wherever we put it) when the slab is
            // resized.
            #[cfg(feature = "gc")]
            get_interned_func_ref(
                vmctx: vmctx,
                func_ref_id: i32,
                module_interned_type_index: i32
            ) -> pointer;

            // Builtin implementation of the `array.new_data` instruction.
            #[cfg(feature = "gc")]
            array_new_data(
                vmctx: vmctx,
                array_interned_type_index: i32,
                data_index: i32,
                data_offset: i32,
                len: i32
            ) -> reference;

            // Builtin implementation of the `array.new_elem` instruction.
            #[cfg(feature = "gc")]
            array_new_elem(
                vmctx: vmctx,
                array_interned_type_index: i32,
                elem_index: i32,
                elem_offset: i32,
                len: i32
            ) -> reference;

            // Builtin implementation of the `array.copy` instruction.
            #[cfg(feature = "gc")]
            array_copy(
                vmctx: vmctx,
                dst_array_interned_type_index: i32,
                dst_array: reference,
                dst_index: i32,
                src_array_interned_type_index: i32,
                src_array: reference,
                src_index: i32,
                len: i32
            );

            // Builtin implementation of the `array.init_data` instruction.
            #[cfg(feature = "gc")]
            array_init_data(
                vmctx: vmctx,
                array_interned_type_index: i32,
                array: reference,
                dst_index: i32,
                data_index: i32,
                data_offset: i32,
                len: i32
            );

            // Builtin implementation of the `array.init_elem` instruction.
            #[cfg(feature = "gc")]
            array_init_elem(
                vmctx: vmctx,
                array_interned_type_index: i32,
                array: reference,
                dst: i32,
                elem_index: i32,
                src: i32,
                len: i32
            );

            // Returns an index for Wasm's `table.grow` instruction for GC references.
            #[cfg(feature = "gc")]
            table_grow_gc_ref(vmctx: vmctx, table: i32, delta: i64, init: reference) -> pointer;

            // Returns an index for Wasm's `table.fill` instruction for GC references.
            #[cfg(feature = "gc")]
            table_fill_gc_ref(vmctx: vmctx, table: i32, dst: i64, val: reference, len: i64);

            // Raises an unconditional trap.
            trap(vmctx: vmctx, code: u8);

            // Implementation of `i{32,64}.trunc_f{32,64}_{u,s}` when host trap
            // handlers are disabled. These will raise a trap if necessary. Note
            // that f32 inputs are always converted to f64 as the argument. Also
            // note that the signed-ness of the result is not reflected in the
            // type here.
            f64_to_i64(vmctx: vmctx, float: f64) -> i64;
            f64_to_u64(vmctx: vmctx, float: f64) -> i64;
            f64_to_i32(vmctx: vmctx, float: f64) -> i32;
            f64_to_u32(vmctx: vmctx, float: f64) -> i32;

            // Creates a new continuation from a funcref.
            tc_cont_new(vmctx: vmctx, r: pointer, param_count: i32, result_count: i32) -> pointer;

            // Sets the tag return values of `child_contref` to those of `parent_contref`.
            // This is implemented by exchanging the pointers to the underlying buffers.
            // `child_contref` must not currently have a tag return value buffer.
            // `parent_contref` may or may not have one.
            tc_cont_ref_forward_tag_return_values_buffer(vmctx: vmctx, parent_contref: pointer, child_contref : pointer);

            // TODO
            tc_drop_cont_ref(vmctx: vmctx, contref: pointer);

            // General-purpose allocation. Only used by stack-switching
            // code at the moment.
            tc_allocate(vmctx: vmctx, size: i64, align: i64) -> pointer;
            // General-purpose deallocation. Only used by stack-switching
            // code at the moment.
            tc_deallocate(vmctx: vmctx, ptr: pointer, size: i64, align: i64);
            // General-purpose reallocation without preserving existing data. Concretely, behaves like
            // deallocate followed by allocate.
            // The only difference is that if `old_size` is 0, then we assume that ptr does not point to allocated memory
            // and do not actually deallocate.
            // `old_size` must be smaller than `new_size`
            tc_reallocate(vmctx: vmctx, ptr: pointer, old_size: i64, new_size: i64, align: i64) -> pointer;

            // TC baseline
            // Baseline resume
            tc_baseline_resume(vmctx: vmctx, contref: pointer) -> i32;
            // Baseline suspend
            tc_baseline_suspend(vmctx: vmctx, tag: i32);
            // Like suspend, but forwards handling.
            tc_baseline_forward(vmctx: vmctx, tag: i32, subcont: pointer);
            // Baseline cont.new
            tc_baseline_cont_new(vmctx: vmctx, r: pointer, param_count: i64, result_count: i64) -> pointer;
            // Baseline continuation drop
            tc_baseline_drop_continuation_reference(vmctx: vmctx, r: pointer);
            // Baseline continuation arguments pointer
            tc_baseline_continuation_arguments_ptr(vmctx: vmctx, r: pointer, nargs: i64) -> pointer;
            // Baseline continuation values pointer
            tc_baseline_continuation_values_ptr(vmctx: vmctx, r: pointer) -> pointer;
            // Baseline continuation clear arguments
            tc_baseline_clear_arguments(vmctx: vmctx, r: pointer);
            // Baseline suspend buffer pointer.
            tc_baseline_get_payloads_ptr(vmctx: vmctx, nargs: i64) -> pointer;
            // Baseline clear suspend buffer.
            tc_baseline_clear_payloads(vmctx: vmctx);
            // Baseline get current continuation.
            tc_baseline_get_current_continuation(vmctx: vmctx) -> pointer;

            // General-purpose printing functions.
            //
            // Prints a string. Note that we transfer the string not
            // as C strings, but as 'static str, represented as a
            // pointer and a length.
            tc_print_str(vmctx: vmctx, s: pointer, len : i64);
            // TODO
            tc_print_int(vmctx: vmctx, arg : i64);
            // TODO
            tc_print_pointer(vmctx: vmctx, arg : pointer);

            // Returns an index for Wasm's `table.grow` instruction
            // for `contobj`s.  Note that the initial
            // Option<VMContObj> (i.e., the value to fill the new
            // slots with) is split into two arguments: The underlying
            // continuation reference and the revision count.  To
            // denote the continuation being `None`, `init_contref`
            // may be 0.
            table_grow_cont_obj(vmctx: vmctx, table: i32, delta: i64, init_contref: pointer, init_revision : i64) -> pointer;
            // `value_contref` and `value_revision` together encode
            // the Option<VMContObj>, as in previous libcall.
            table_fill_cont_obj(vmctx: vmctx, table: i32, dst: i64, value_contref: pointer, value_revision : i64, len: i64);
        }
    };
}

/// An index type for builtin functions.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
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

            /// Returns a symbol name for this builtin.
            pub fn name(&self) -> &'static str {
                $(
                    $( #[$attr] )*
                    if *self == BuiltinFunctionIndex::$name() {
                        return stringify!($name);
                    }
                )*
                unreachable!()
            }
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
        #[allow(missing_docs)]
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
