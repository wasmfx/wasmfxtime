use cranelift_codegen::ir;
use cranelift_codegen::ir::InstBuilder;

use cranelift_frontend::FunctionBuilder;

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

struct TaggedPointer;

type Uintptr = u64;
const UINTPTR_MAX: u64 = 18_446_744_073_709_551_615u64;

impl<'a> TaggedPointer {
    const HB_TAG_BITS: u64 = 16;
    const HB_POINTER_BITS: u64 = 48;
    const HB_POINTER_MASK: Uintptr = (UINTPTR_MAX >> Self::HB_TAG_BITS);

    pub fn untag(
        _env: &mut crate::func_environ::FuncEnvironment<'a>,
        builder: &mut FunctionBuilder,
        ptr: ir::Value,
    ) -> (ir::Value, ir::Value) {
        let tag = builder.ins().ushr_imm(ptr, Self::HB_POINTER_BITS as i64);
        let unmasked = builder.ins().band_imm(ptr, Self::HB_POINTER_MASK as i64);
        (tag, unmasked)
    }

    pub fn with_tag(
        _env: &mut crate::func_environ::FuncEnvironment<'a>,
        builder: &mut FunctionBuilder,
        tag: ir::Value,
        ptr: ir::Value,
    ) -> ir::Value {
        let tag = builder.ins().ishl_imm(tag, Self::HB_POINTER_BITS as i64);
        builder.ins().bor(ptr, tag)
    }
}

pub(crate) fn disassemble_contobj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
) -> (ir::Value, ir::Value) {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        let zero = builder.ins().iconst(cranelift_codegen::ir::types::I64, 0);
        (zero, contobj)
    } else {
        TaggedPointer::untag(env, builder, contobj)
    }
}

pub(crate) fn assemble_contobj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    count: ir::Value,
    contref_addr: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        contref_addr
    } else {
        TaggedPointer::with_tag(env, builder, count, contref_addr)
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
