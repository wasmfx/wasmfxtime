use cranelift_codegen::ir;
use cranelift_codegen::ir::InstBuilder;

use cranelift_frontend::FunctionBuilder;

#[allow(unused_macros)]
macro_rules! call_builtin {
    ( $builder:ident, $env:ident, $f:ident( $($args:expr),* ) ) => (
        let _fname = $env.builtin_functions.$f(&mut $builder.func);
        let _vmctx_libcall_arg = $env.vmctx_val(&mut $builder.cursor());
        let _call_inst = $builder.ins().call(_fname, &[_vmctx_libcall_arg, $( $args ), * ]);
    );
    ( $builder:ident, $env:ident, let $name:ident = $f:ident( $($args:expr),* ) )=> (
        let _fname = $env.builtin_functions.$f(&mut $builder.func);
        let _vmctx_libcall_arg = $env.vmctx_val(&mut $builder.cursor());
        let _call_inst = $builder.ins().call(_fname, &[_vmctx_libcall_arg, $( $args ), * ]);
        let $name = *$builder.func.dfg.inst_results(_call_inst).first().unwrap();
    );
}

#[allow(unused_imports)]
pub(crate) use call_builtin;

/// TODO
pub(crate) fn typed_continuations_cont_Xobj_get_cont_Xref<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contXobj: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        // The "contXobj" is a contXref already
        return contXobj;
    } else {
        call_builtin!(builder, env, let result = tc_cont_Xobj_get_cont_Xref(contXobj));
        return result;
    }
}

pub(crate) fn typed_continuations_new_cont_Xobj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contXref_addr: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        return contXref_addr;
    } else {
        call_builtin!(builder, env, let result = tc_new_cont_Xobj(contXref_addr));
        return result;
    }
}

/// TODO
pub(crate) fn typed_continuations_drop_cont_Xref<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contXref: ir::Value,
) {
    call_builtin!(builder, env, tc_drop_cont_Xref(contXref));
}
