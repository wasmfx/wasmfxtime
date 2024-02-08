use cranelift_codegen::ir;

use cranelift_frontend::FunctionBuilder;

use wasmtime_environ::BuiltinFunctionIndex;

#[allow(unused_macros)]
macro_rules! generate_builtin_call {
    ($env : ident, $builder: ident, $builtin_name: ident, $args: expr) => {{
        let index = BuiltinFunctionIndex::$builtin_name();
        let sig = $env
            .builtin_function_signatures
            .$builtin_name(&mut $builder.func);
        let args = $args.to_vec();
        $env.generate_builtin_call($builder, index, sig, args)
    }};
}

#[allow(unused_macros)]
macro_rules! generate_builtin_call_no_return_val {
    ($env : ident, $builder: ident, $builtin_name: ident, $args: expr) => {{
        let index = BuiltinFunctionIndex::$builtin_name();
        let sig = $env
            .builtin_function_signatures
            .$builtin_name(&mut $builder.func);
        let args = $args.to_vec();
        $env.generate_builtin_call_no_return_val($builder, index, sig, args)
    }};
}

#[allow(unused_imports)]
pub(crate) use generate_builtin_call;
#[allow(unused_imports)]
pub(crate) use generate_builtin_call_no_return_val;

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
        let (_vmctx, contobj) =
            generate_builtin_call!(env, builder, tc_cont_ref_get_cont_obj, [contref]);
        return contobj;
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
        let (_vmctx, contref) =
            generate_builtin_call!(env, builder, tc_new_cont_ref, [contobj_addr]);
        return contref;
    }
}

/// TODO
pub(crate) fn typed_continuations_drop_cont_obj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
) {
    generate_builtin_call_no_return_val!(env, builder, tc_drop_cont_obj, [contobj]);
}
