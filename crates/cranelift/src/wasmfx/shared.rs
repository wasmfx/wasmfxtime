use cranelift_codegen::ir;
use cranelift_codegen::ir::InstBuilder;

use cranelift_frontend::FunctionBuilder;

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
        let cont_ref_get_cont_obj = env
            .builtin_functions
            .tc_cont_ref_get_cont_obj(&mut builder.func);
        let vmctx = env.vmctx_val(&mut builder.cursor());
        let call_inst = builder.ins().call(cont_ref_get_cont_obj, &[vmctx, contref]);
        let result = *builder.func.dfg.inst_results(call_inst).first().unwrap();
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
        let new_cont_ref = env.builtin_functions.tc_new_cont_ref(&mut builder.func);
        let vmctx = env.vmctx_val(&mut builder.cursor());
        let call_inst = builder.ins().call(new_cont_ref, &[vmctx, contobj_addr]);
        let result = *builder.func.dfg.inst_results(call_inst).first().unwrap();
        return result;
    }
}

/// TODO
pub(crate) fn typed_continuations_drop_cont_obj<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
) {
    let cont_drop_obj = env.builtin_functions.tc_drop_cont_obj(&mut builder.func);
    let vmctx = env.vmctx_val(&mut builder.cursor());
    builder.ins().call(cont_drop_obj, &[vmctx, contobj]);
}
