use super::shared;

use crate::translate::{FuncEnvironment, FuncTranslationState};
use crate::wasmfx::shared::call_builtin;
use cranelift_codegen::ir;
use cranelift_codegen::ir::condcodes::*;
use cranelift_codegen::ir::types::*;
use cranelift_codegen::ir::InstBuilder;
use cranelift_frontend::{FunctionBuilder, Switch};
use wasmtime_environ::PtrSize;
use wasmtime_environ::{WasmResult, WasmValType};

#[cfg_attr(not(feature = "wasmfx_baseline"), allow(unused_imports))]
#[cfg_attr(feature = "wasmfx_no_baseline", allow(unused_imports))]
pub(crate) use shared::{assemble_contobj, disassemble_contobj, vm_contobj_type};

fn get_revision<'a>(
    _env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contref: ir::Value,
) -> ir::Value {
    let mem_flags = ir::MemFlags::trusted();
    builder.ins().load(I64, mem_flags, contref, 0)
}

fn compare_revision_and_increment<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contref: ir::Value,
    witness: ir::Value,
) -> ir::Value {
    let mem_flags = ir::MemFlags::trusted();
    let revision = get_revision(env, builder, contref);

    let evidence = builder.ins().icmp(IntCC::Equal, witness, revision);
    builder
        .ins()
        .trapz(evidence, crate::TRAP_CONTINUATION_ALREADY_CONSUMED);

    let revision_plus1 = builder.ins().iadd_imm(revision, 1);
    builder.ins().store(mem_flags, revision_plus1, contref, 0);
    revision_plus1
}

fn typed_continuations_load_payloads<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    valtypes: &[ir::Type],
) -> Vec<ir::Value> {
    let mut values = vec![];

    if valtypes.len() > 0 {
        // Retrieve the pointer to the suspend buffer.
        let nargs = builder.ins().iconst(I64, 0);
        call_builtin!(builder, env, let payloads_ptr = tc_baseline_get_payloads_ptr(nargs));
        // Load payloads.
        let memflags = ir::MemFlags::trusted();
        let mut offset = 0;
        for valtype in valtypes {
            let val = builder.ins().load(*valtype, memflags, payloads_ptr, offset);
            values.push(val);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }
        // Clear the payloads buffer
        call_builtin!(builder, env, tc_baseline_clear_payloads());
    }
    values
}

pub(crate) fn typed_continuations_load_tag_return_values<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contref: ir::Value,
    valtypes: &[WasmValType],
) -> Vec<ir::Value> {
    let _memflags = ir::MemFlags::trusted();
    let values = vec![];

    if valtypes.len() > 0 {
        // Retrieve the pointer to the arguments' buffer.
        let nargs = builder.ins().iconst(I64, 0);
        call_builtin!(
            builder,
            env,
            let args_ptr =
                tc_baseline_continuation_arguments_ptr(
                contref,
                nargs
            )
        );

        // Load arguments.
        let mut args = vec![];
        let mut offset = 0;
        let memflags = ir::MemFlags::trusted();
        for valtype in valtypes {
            let val = builder.ins().load(
                crate::value_type(env.isa, *valtype),
                memflags,
                args_ptr,
                offset,
            );
            args.push(val);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }
        debug_assert!(valtypes.len() == args.len());

        // Clear the arguments buffer
        call_builtin!(builder, env, tc_baseline_clear_arguments(contref));

        return args;
    }

    values
}

/// TODO
pub(crate) fn typed_continuations_store_resume_args<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    values: &[ir::Value],
    _remaining_arg_count: ir::Value,
    contref: ir::Value,
) {
    if values.len() > 0 {
        // Retrieve the pointer to the arguments buffer.
        let nargs = builder.ins().iconst(I64, values.len() as i64);
        call_builtin!(
            builder,
            env,
            let args_ptr =
                tc_baseline_continuation_arguments_ptr(
                contref,
                nargs
            )
        );

        // Store arguments.
        let memflags = ir::MemFlags::trusted();
        let mut offset = 0;
        for arg in values {
            builder.ins().store(memflags, *arg, args_ptr, offset);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }
    }
}

//TODO(frank-emrich) Consider removing `valtypes` argument, as values are inherently typed
pub(crate) fn typed_continuations_store_payloads<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    values: &[ir::Value],
) {
    if values.len() > 0 {
        // Retrieve the pointer to the payloads buffer.
        let nargs = builder.ins().iconst(I64, values.len() as i64);
        call_builtin!(builder, env, let payloads_ptr = tc_baseline_get_payloads_ptr(nargs));
        // Store arguments.
        let memflags = ir::MemFlags::trusted();
        let mut offset = 0;
        for arg in values {
            builder.ins().store(memflags, *arg, payloads_ptr, offset);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }
    }
}

pub(crate) fn typed_continuations_load_continuation_reference<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
) -> ir::Value {
    call_builtin!(builder, env, let result = tc_baseline_get_current_continuation());
    return result;
}

pub(crate) fn translate_resume<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    type_index: u32,
    resumee_obj: ir::Value,
    resume_args: &[ir::Value],
    resumetable: &[(u32, ir::Block)],
) -> Vec<ir::Value> {
    // The resume instruction is by far the most involved
    // instruction to compile as it is responsible for both
    // continuation application and effect dispatch.
    //
    // We store the continuation arguments, continuation return
    // values, and suspension payloads on objects accessible
    // inside the libcall context.
    //
    // Here we translate a resume instruction into several basic
    // blocks as follows:
    //
    //        prelude_block
    //              |
    //              |
    //        resume_block <--------
    //        |          |          \
    //        |          |          |
    // return_block   handle_block  |
    //                     |        |
    //                     |        |
    //                forward_block /
    //
    // * prelude_block pushes the continuation arguments onto the
    //   buffer in the libcall context.
    // * resume_block continues a given `resumee_ref`. It jumps to
    //   return_block on ordinary return and to `handle_block` on
    //   a suspension
    // * return_block reads the return values from the libcall
    //   context.
    // * handle_block dispatches on a tag provided by the
    //   resume_block to an associated user-defined block. If
    //   there is no suitable user-defined block, then it jumps to
    //   the forward_block.
    // * forward_block dispatches the handling of a given tag to
    //   the ambient context. Once control returns it jumps to the
    //   resume_block to continue continuation at the suspension
    //   site.
    let resume_block = builder.create_block();
    let return_block = builder.create_block();
    let handle_block = builder.create_block();
    let forwarding_block = builder.create_block();

    // Prelude: Push the continuation arguments.
    let next_revision = {
        let (witness, resumee_fiber) = disassemble_contobj(env, builder, resumee_obj);
        let next_revision = compare_revision_and_increment(env, builder, resumee_fiber, witness);
        if resume_args.len() > 0 {
            let nargs = builder.ins().iconst(I64, resume_args.len() as i64);

            // Load the arguments pointer
            call_builtin!(
                builder,
                env,
                let args_ptr =
                    tc_baseline_continuation_arguments_ptr(
                    resumee_fiber,
                    nargs
                )
            );

            // Append arguments.
            let memflags = ir::MemFlags::trusted();
            let mut offset = 0;
            for arg in resume_args {
                builder.ins().store(memflags, *arg, args_ptr, offset);
                offset += env.offsets.ptr.maximum_value_size() as i32;
            }
        }

        builder.ins().jump(resume_block, &[resumee_fiber]);
        next_revision
    };

    // Resume block: here we continue the (suspended) resumee.
    let (tag, resumee_fiber) = {
        builder.switch_to_block(resume_block);
        builder.append_block_param(resume_block, env.pointer_type());

        // The raw continuation fiber
        let resumee_fiber = builder.block_params(resume_block)[0];

        // Load the builtin continuation resume function.
        call_builtin!(builder, env, let result = tc_baseline_resume(resumee_fiber));

        // The result encodes whether the return happens via ordinary
        // means or via a suspend. If the high bit is set, then it is
        // interpreted as the return happened via a suspend, and the
        // remainder of the integer is to be interpreted as the index
        // of the control tag that was supplied to the suspend.
        let signal_mask = 0xf000_0000;
        let inverse_signal_mask = 0x0fff_ffff;
        let signal = builder.ins().band_imm(result, signal_mask);
        let tag = builder.ins().band_imm(result, inverse_signal_mask);

        // Test the signal bit.
        let is_zero = builder.ins().icmp_imm(IntCC::Equal, signal, 0);

        // Jump to the return block if the signal is 0, otherwise
        // jump to the suspend block.
        builder
            .ins()
            .brif(is_zero, return_block, &[], handle_block, &[]);

        // We do not seal this block, yet, because the effect forwarding block has a back edge to it
        (tag, resumee_fiber)
    };

    // Now we construct the handling block.
    //
    // Strategy:
    //
    // We encode the resume table as a sparse switch table, where
    // each `(tag, label)` pair in the resume table translates to
    // a "case tag: br label" in the switch table.
    //
    // The first occurs of a particular tag `t` shadows every
    // other occurrence of the same tag `t` in the resumetable,
    // meaning we should only emit code for the first such `t`.
    //
    // Effect forwarding is handled by the default/fallthrough
    // case of the switch.

    // First, initialise the switch structure.
    let mut switch = Switch::new();
    // Second, we consume the resume table entry-wise.
    let mut case_blocks = vec![];
    let mut tag_seen = std::collections::HashSet::new(); // Used to keep track of tags
    for &(tag, label) in resumetable {
        // Skip if this `tag` has been seen previously.
        if !tag_seen.insert(tag) {
            continue;
        }
        let case = builder.create_block();
        switch.set_entry(tag as u128, case);
        builder.switch_to_block(case);

        // Load and push payloads.
        let param_types: Vec<ir::Type> = env
            .tag_params(tag)
            .iter()
            .map(|wty| crate::value_type(env.isa, *wty))
            .collect();
        // NOTE(dhil): For the baseline, `load_payloads` actually
        // moves the payloads, i.e. consumes the payloads buffer
        // entirely.
        let mut args = typed_continuations_load_payloads(env, builder, &param_types);

        // Create and push the continuation object.
        let resumee_obj = assemble_contobj(env, builder, next_revision, resumee_fiber);
        args.push(resumee_obj);

        // Finally, emit the jump to `label`.
        builder.ins().jump(label, &args);
        case_blocks.push(case);
    }

    // Forwarding block.
    {
        builder.switch_to_block(forwarding_block);

        // Load the builtin forwarding function.
        call_builtin!(builder, env, tc_baseline_forward(tag, resumee_fiber));

        builder.ins().jump(resume_block, &[resumee_fiber]);
        builder.seal_block(resume_block);
    }

    // Emit the switch.
    {
        builder.switch_to_block(handle_block);
        switch.emit(builder, tag, forwarding_block);
        builder.seal_block(handle_block);
        builder.seal_block(forwarding_block);

        for case_block in case_blocks {
            builder.seal_block(case_block);
        }
    }

    // Return block.
    {
        builder.switch_to_block(return_block);
        builder.seal_block(return_block);

        // Load the values pointer.
        call_builtin!(
            builder,
            env,
            let vals_ptr =
                tc_baseline_continuation_values_ptr(
                resumee_fiber
            )
        );

        // Load and push the return values.
        let returns = env.continuation_returns(type_index);
        let mut values = vec![];
        let mut offset = 0;
        let memflags = ir::MemFlags::trusted();
        for valtype in returns {
            let val = builder.ins().load(
                crate::value_type(env.isa, *valtype),
                memflags,
                vals_ptr,
                offset,
            );
            values.push(val);
            offset += env.offsets.ptr.maximum_value_size() as i32;
        }

        // Free the continuation.
        call_builtin!(
            builder,
            env,
            tc_baseline_drop_continuation_reference(resumee_fiber)
        );

        return values;
    }
}

pub(crate) fn translate_cont_bind<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    contobj: ir::Value,
    args: &[ir::Value],
    remaining_arg_count: usize,
) -> ir::Value {
    let (witness, contref) = disassemble_contobj(env, builder, contobj);
    let revision = compare_revision_and_increment(env, builder, contref, witness);
    let remaining_arg_count = builder.ins().iconst(I32, remaining_arg_count as i64);
    typed_continuations_store_resume_args(env, builder, args, remaining_arg_count, contref);
    assemble_contobj(env, builder, revision, contref)
}

pub(crate) fn translate_cont_new<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    _state: &FuncTranslationState,
    func: ir::Value,
    arg_types: &[WasmValType],
    return_types: &[WasmValType],
) -> WasmResult<ir::Value> {
    // Load the builtin continuation allocation function.
    let nargs = builder.ins().iconst(I64, arg_types.len() as i64);
    let nreturns = builder.ins().iconst(I64, return_types.len() as i64);
    call_builtin!(builder, env, let contref = tc_baseline_cont_new(func, nargs, nreturns));
    let revision = get_revision(env, builder, contref);
    let contobj = assemble_contobj(env, builder, revision, contref);
    Ok(contobj)
}

pub(crate) fn translate_suspend<'a>(
    env: &mut crate::func_environ::FuncEnvironment<'a>,
    builder: &mut FunctionBuilder,
    tag_index: u32,
    suspend_args: &[ir::Value],
    tag_return_types: &[WasmValType],
) -> Vec<ir::Value> {
    let tag_index_val = builder.ins().iconst(I32, tag_index as i64);
    typed_continuations_store_payloads(env, builder, suspend_args);
    call_builtin!(builder, env, tc_baseline_suspend(tag_index_val));
    let contref = typed_continuations_load_continuation_reference(env, builder);

    let return_values =
        typed_continuations_load_tag_return_values(env, builder, contref, tag_return_types);

    return_values
}
