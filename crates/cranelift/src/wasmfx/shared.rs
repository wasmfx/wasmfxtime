use cranelift_codegen::ir;
use cranelift_codegen::ir::InstBuilder;

use cranelift_frontend::FunctionBuilder;
use cranelift_wasm::TargetEnvironment;

#[allow(unused_macros)]
macro_rules! call_builtin {
    ( $builder:ident, $env:ident, $f:ident( $($args:expr),* ) ) => (
        {
            let fname = $env.builtin_functions.$f(&mut $builder.func);
            let vmctx = $env.vmctx_val(&mut $builder.cursor());
            $builder.ins().call(fname, &[vmctx, $( $args ), * ]);
        }
    );
    ( $builder:ident, $env:ident, let $name:ident = $f:ident( $($args:expr),* ) )=> (
        let $name = {
            let fname = $env.builtin_functions.$f(&mut $builder.func);
            let vmctx = $env.vmctx_val(&mut $builder.cursor());
            let call_inst = $builder.ins().call(fname, &[vmctx, $( $args ), * ]);
            *$builder.func.dfg.inst_results(call_inst).first().unwrap()
        };
    );
}

#[allow(unused_imports)]
pub(crate) use call_builtin;

/// The Cranelfift type used to represent all of the following:
/// - wasm values of type `(ref null $ct)` and `(ref $ct)`
/// - equivalenty: runtime values of type `Option<VMContObj>` and `VMContObj`
pub(crate) fn vm_contobj_type(pointer_type: ir::Type) -> ir::Type {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        // If linearity checks are disabled, a `VMContObj` is just a pointer
        // to the underlying `VMContRef`.
        // For consistency with the fat pointer case, we use I32/I64 here
        // instead of RI32/I64 (which are used for other reference types)
        pointer_type
    } else {
        // If linearity checks are enabled, a `VMContObj` is a fat pointer
        // consisting of a pointer to `VMContRef` and a 64 bit sequence
        // counter.

        // Naturally, you may wonder why we don't use any of the following
        // types instead:
        //
        // - I128: We can't use this type, because cranelift only allows
        // using this type for parameters/return values if the setting
        // `enable_llvm_abi_extensions` is enabled, which is not allowed
        // when using cranelift for wasmtime.
        //
        // - I64X2: If we have to use a 128 bit vector type for our
        // continuations in Cranelift, the most reasonable choice would be
        // I64X2. After all, our fat pointers consist of an (up to) 64bit
        // pointer and a 64 bit counter. The reason why we can't use this
        // type is that wasmtime assumes that all wasm SIMD values have the
        // same Cranelift type, namely I8X16. As a result,
        // [cranelift_wasm::code_translator] liberally inserts `bitcast`
        // instructions to turn all vector types it sees into the canonical
        // type I8X16. Thus, if we used I64X2 for our continuation values
        // in wasm, this canonicalization, intended for actual SIMD wasm
        // values, would break our code. `bitcast`-ing between I64X2 and
        // I16X8 is a noop, so this has no performance impact.

        // NOTE(frank-emrich) We currently only care about little endian
        // platforms. The internal layout of the vector is reflected by
        // this, it is identical to what happens if you do a 128bit vector
        // load of a `Optional<VMContObj>` on a little endian platform: Its
        // 64 LSBs contain the revision counter, its 64MSBs contain the
        // `VMContRef` pointer.
        ir::types::I8X16
    }
}

/// Unless linearity checks disabled, turns a (possibly null reference to a)
/// continuation object into a tuple (revision, contref_ptr).
/// If `contobj` denotes a wasm null reference, the contref_ptr part may be a null pointer.
pub(crate) fn disassemble_contobj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
) -> (ir::Value, ir::Value) {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        let zero = builder.ins().iconst(cranelift_codegen::ir::types::I64, 0);
        (zero, contobj)
    } else {
        debug_assert_eq!(
            builder.func.dfg.value_type(contobj),
            vm_contobj_type(env.pointer_type())
        );
        let flags = ir::MemFlags::new().with_endianness(ir::Endianness::Little);
        let contobj = builder.ins().bitcast(ir::types::I64X2, flags, contobj);
        let revision_counter = builder.ins().extractlane(contobj, 0);
        let contref = builder.ins().extractlane(contobj, 1);
        debug_assert_eq!(builder.func.dfg.value_type(contref), ir::types::I64);
        debug_assert_eq!(
            builder.func.dfg.value_type(revision_counter),
            ir::types::I64
        );
        // TODO(frank-emrich) On 32bit platforms, need to ireduce contref to env.pointer_type()
        (revision_counter, contref)
    }
}

/// Constructs a continuation object from a given contref and revision pointer.
/// The contref_addr may be 0, to indicate that we want to build a wasm null reference.
pub(crate) fn assemble_contobj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    revision_counter: ir::Value,
    contref_addr: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        contref_addr
    } else {
        // TODO(frank-emrich) This check assumes env.pointer_type() == I64
        debug_assert_eq!(builder.func.dfg.value_type(contref_addr), ir::types::I64);
        debug_assert_eq!(
            builder.func.dfg.value_type(revision_counter),
            ir::types::I64
        );

        let lower = builder
            .ins()
            .scalar_to_vector(ir::types::I64X2, revision_counter);
        let contobj = builder.ins().insertlane(lower, contref_addr, 1);

        let flags = ir::MemFlags::new().with_endianness(ir::Endianness::Little);
        let contobj = builder
            .ins()
            .bitcast(vm_contobj_type(env.pointer_type()), flags, contobj);
        contobj
    }
}

/// TODO
pub(crate) fn typed_continuations_drop_cont_ref<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contref: ir::Value,
) {
    call_builtin!(builder, env, tc_drop_cont_ref(contref));
}
