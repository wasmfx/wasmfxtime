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
pub(crate) fn typed_continuations_cont_obj_get_cont_ref<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        // The "contobj" is a contref already
        return contobj;
    } else {
        call_builtin!(builder, env, let result = tc_cont_obj_get_cont_ref(contobj));
        return result;
    }
}

pub(crate) fn typed_continuations_new_cont_obj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contref_addr: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        return contref_addr;
    } else {
        call_builtin!(builder, env, let result = tc_new_cont_obj(contref_addr));
        return result;
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
