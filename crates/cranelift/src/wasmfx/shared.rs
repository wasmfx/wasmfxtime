use cranelift_codegen::ir;
use cranelift_codegen::ir::InstBuilder;

use cranelift_frontend::FunctionBuilder;

#[allow(unused_macros)]
macro_rules! call_builtin {
     ( $builder:ident, $env:ident, $f:ident ) => (
        let _fname = $env.builtin_functions.$f(&mut $builder.func);
        let _vmctx_libcall_arg = $env.vmctx_val(&mut $builder.cursor());
        let _call_inst = $builder.ins().call(_fname, &[_vmctx_libcall_arg]);
    );
    ( $builder:ident, $env:ident, $f:ident, $($args:expr),* ) => (
        let _fname = $env.builtin_functions.$f(&mut $builder.func);
        let _vmctx_libcall_arg = $env.vmctx_val(&mut $builder.cursor());
        let _call_inst = $builder.ins().call(_fname, &[_vmctx_libcall_arg, $( $args ), * ]);
    );
    ( $name:ident = ($builder:ident, $env:ident, $f:ident ) ) => (
        let _fname = $env.builtin_functions.$f(&mut $builder.func);
        let _vmctx_libcall_arg = $env.vmctx_val(&mut $builder.cursor());
        let _call_inst = $builder.ins().call(_fname, &[_vmctx_libcall_arg]);
        let $name = *$builder.func.dfg.inst_results(_call_inst).first().unwrap();
    );
    ( $name:ident = ($builder:ident, $env:ident, $f:ident, $($args:expr),*) ) => (
        let _fname = $env.builtin_functions.$f(&mut $builder.func);
        let _vmctx_libcall_arg = $env.vmctx_val(&mut $builder.cursor());
        let _call_inst = $builder.ins().call(_fname, &[_vmctx_libcall_arg, $( $args ), * ]);
        let $name = *$builder.func.dfg.inst_results(_call_inst).first().unwrap();
    );
}

#[allow(unused_imports)]
pub(crate) use call_builtin;

/// TODO
pub(crate) fn typed_continuations_cont_ref_get_cont_obj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contref: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        // The "contref" is a contobj already
        return contref;
    } else {
        call_builtin!(result = (builder, env, tc_cont_ref_get_cont_obj, contref));
        return result;
    }
}

pub(crate) fn typed_continuations_new_cont_ref<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj_addr: ir::Value,
) -> ir::Value {
    if cfg!(feature = "unsafe_disable_continuation_linearity_check") {
        return contobj_addr;
    } else {
        call_builtin!(result = (builder, env, tc_new_cont_ref, contobj_addr));
        return result;
    }
}

/// TODO
pub(crate) fn typed_continuations_drop_cont_obj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
) {
    call_builtin!(builder, env, tc_drop_cont_obj, contobj);
}
