//! Backtrace and stack walking functionality for Wasm.
//!
//! Walking the Wasm stack is comprised of
//!
//! 1. identifying sequences of contiguous Wasm frames on the stack
//!    (i.e. skipping over native host frames), and
//!
//! 2. walking the Wasm frames within such a sequence.
//!
//! To perform (1) we maintain the entry stack pointer (SP) and exit frame
//! pointer (FP) and program counter (PC) each time we call into Wasm and Wasm
//! calls into the host via trampolines (see
//! `crates/runtime/src/trampolines`). The most recent entry is stored in
//! `VMRuntimeLimits` and older entries are saved in `CallThreadState`. This
//! lets us identify ranges of contiguous Wasm frames on the stack.
//!
//! To solve (2) and walk the Wasm frames within a region of contiguous Wasm
//! frames on the stack, we configure Cranelift's `preserve_frame_pointers =
//! true` setting. Then we can do simple frame pointer traversal starting at the
//! exit FP and stopping once we reach the entry SP (meaning that the next older
//! frame is a host frame).

use wasmtime_continuations::StackChain;

use crate::arch;
use crate::{
    traphandlers::{tls, CallThreadState},
    VMRuntimeLimits,
};
use std::ops::ControlFlow;

/// A WebAssembly stack trace.
#[derive(Debug)]
pub struct Backtrace(Vec<Frame>);

/// A stack frame within a Wasm stack trace.
#[derive(Debug)]
pub struct Frame {
    pc: usize,
    fp: usize,
}

impl Frame {
    /// Get this frame's program counter.
    pub fn pc(&self) -> usize {
        self.pc
    }

    /// Get this frame's frame pointer.
    pub fn fp(&self) -> usize {
        self.fp
    }
}

impl Backtrace {
    /// Returns an empty backtrace
    pub fn empty() -> Backtrace {
        Backtrace(Vec::new())
    }

    /// Capture the current Wasm stack in a backtrace.
    pub fn new(limits: *const VMRuntimeLimits) -> Backtrace {
        tls::with(|state| match state {
            Some(state) => unsafe { Self::new_with_trap_state(limits, state, None) },
            None => Backtrace(vec![]),
        })
    }

    /// Capture the current Wasm stack trace.
    ///
    /// If Wasm hit a trap, and we calling this from the trap handler, then the
    /// Wasm exit trampoline didn't run, and we use the provided PC and FP
    /// instead of looking them up in `VMRuntimeLimits`.
    pub(crate) unsafe fn new_with_trap_state(
        limits: *const VMRuntimeLimits,
        state: &CallThreadState,
        trap_pc_and_fp: Option<(usize, usize)>,
    ) -> Backtrace {
        let mut frames = vec![];
        Self::trace_with_trap_state(limits, state, trap_pc_and_fp, |frame| {
            frames.push(frame);
            ControlFlow::Continue(())
        });
        Backtrace(frames)
    }

    /// Walk the current Wasm stack, calling `f` for each frame we walk.
    pub fn trace(limits: *const VMRuntimeLimits, f: impl FnMut(Frame) -> ControlFlow<()>) {
        tls::with(|state| match state {
            Some(state) => unsafe { Self::trace_with_trap_state(limits, state, None, f) },
            None => {}
        });
    }

    /// Walk the current Wasm stack, calling `f` for each frame we walk.
    ///
    /// If Wasm hit a trap, and we calling this from the trap handler, then the
    /// Wasm exit trampoline didn't run, and we use the provided PC and FP
    /// instead of looking them up in `VMRuntimeLimits`.
    pub(crate) unsafe fn trace_with_trap_state(
        limits: *const VMRuntimeLimits,
        state: &CallThreadState,
        trap_pc_and_fp: Option<(usize, usize)>,
        mut f: impl FnMut(Frame) -> ControlFlow<()>,
    ) {
        if cfg!(feature = "typed_continuations_baseline_implementation") {
            if crate::continuation::baseline::has_ever_run_continuation() {
                log::info!("Backtrace generation not supported in baseline implementation once a continuation has been invoked");
                return;
            }
        }

        log::trace!("====== Capturing Backtrace ======");

        // We are only interested in wasm frames, not host frames. Thus, we peel
        // away the first states in this thread's `CallThreadState` chain that
        // do not execute wasm.
        // Note(frank-emrich) I'm not entirely sure if it can ever be the case
        // that `state` is not actually executing wasm. In other words, it may
        // be the case that we always have `Some(state) == first_wasm_state`.
        // Otherwise we would be building a backtrace while executing a host
        // call. But those cannot trap, but only panic and we do not use this
        // function to build backtraces for panics.
        let first_wasm_state = state
            .iter()
            .flat_map(|head| head.iter())
            .skip_while(|state| state.callee_stack_chain.is_none())
            .next();

        let (last_wasm_exit_pc, last_wasm_exit_fp) = match trap_pc_and_fp {
            // If we exited Wasm by catching a trap, then the Wasm-to-host
            // trampoline did not get a chance to save the last Wasm PC and FP,
            // and we need to use the plumbed-through values instead.
            Some((pc, fp)) => {
                assert!(std::ptr::eq(limits, state.limits));
                assert!(first_wasm_state.is_some());
                (pc, fp)
            }
            // Either there is no Wasm currently on the stack, or we exited Wasm
            // through the Wasm-to-host trampoline.
            None => {
                let pc = *(*limits).last_wasm_exit_pc.get();
                let fp = *(*limits).last_wasm_exit_fp.get();
                (pc, fp)
            }
        };

        let first_wasm_state_stack_chain = first_wasm_state
            .map(|state| state.callee_stack_chain.map(|cell| &*(*cell).0.get()))
            .flatten();

        // The first value in `activations` is for the most recently running
        // wasm. We thus provide the stack chain of `first_wasm_state` to
        // traverse the potential continuation stacks. For the subsequent
        // activations, we unconditionally use `None` as the corresponding stack
        // chain. This is justified because only the most recent execution of
        // wasm may execute off the main stack (see comments in
        // `wasmtime::invoke_wasm_and_catch_traps` for details).
        let activations = std::iter::once((
            first_wasm_state_stack_chain,
            last_wasm_exit_pc,
            last_wasm_exit_fp,
            *(*limits).last_wasm_entry_sp.get(),
        ))
        .chain(
            first_wasm_state
                .iter()
                .flat_map(|state| state.iter())
                .filter(|state| std::ptr::eq(limits, state.limits))
                .map(|state| {
                    (
                        None,
                        state.old_last_wasm_exit_pc(),
                        state.old_last_wasm_exit_fp(),
                        state.old_last_wasm_entry_sp(),
                    )
                }),
        )
        .take_while(|&(_chain, pc, fp, sp)| {
            if pc == 0 {
                debug_assert_eq!(fp, 0);
                debug_assert_eq!(sp, 0);
            }
            pc != 0
        });

        for (chain, pc, fp, sp) in activations {
            if let ControlFlow::Break(()) =
                Self::trace_through_continuations(chain, pc, fp, sp, &mut f)
            {
                log::trace!("====== Done Capturing Backtrace (closure break) ======");
                return;
            }
        }

        log::trace!("====== Done Capturing Backtrace (reached end of activations) ======");
    }

    unsafe fn trace_through_continuations(
        chain: Option<&StackChain>,
        pc: usize,
        fp: usize,
        trampoline_sp: usize,
        mut f: impl FnMut(Frame) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        // Handle the stack that is currently running (which may be a
        // continuation or the main stack).
        Self::trace_through_wasm(pc, fp, trampoline_sp, &mut f)?;

        chain.map_or(ControlFlow::Continue(()), |chain| {
            debug_assert_ne!(*chain, StackChain::Absent);

            let stack_limits_iter = chain.clone().into_iter();

            // The very first entry in the stack chain is for what is currently
            // running (which may be a continuation or main stack). However, for
            // the currently running stack, the data in its associated
            // `StackLimits` object is stale (see comment on
            // `wasmtime_continuations::StackChain` for a description of the
            // invariants).
            // That's why we already handled the currently running stack at the
            // beginning of the function, using data directly from the
            // VMRuntimeLimits.
            let remainder = stack_limits_iter.skip(1);

            for (continuation_opt, limits) in remainder {
                let limits = limits.as_ref().unwrap();
                match continuation_opt {
                    Some(continuation) => {
                        let cont = unsafe { &*continuation };
                        let stack_range = (*cont.fiber).stack().range().unwrap();
                        debug_assert!(stack_range.contains(&limits.last_wasm_exit_fp));
                        debug_assert!(stack_range.contains(&limits.last_wasm_entry_sp));
                        // TODO(frank-emrich) Enable this assertion one we stop
                        // zero-ing the stack limit in
                        // `wasmtime_runtime::continuation::resume`
                        //
                        // debug_assert_eq!(stack_range.end, limits.stack_limit);
                    }
                    None => {
                        // reached stack information for main stack
                    }
                }

                Self::trace_through_wasm(
                    limits.last_wasm_exit_pc,
                    limits.last_wasm_exit_fp,
                    limits.last_wasm_entry_sp,
                    &mut f,
                )?
            }
            ControlFlow::Continue(())
        })
    }

    /// Walk through a contiguous sequence of Wasm frames starting with the
    /// frame at the given PC and FP and ending at `trampoline_sp`.
    // TODO(frank-emrich) Implement tracing across continuations.
    unsafe fn trace_through_wasm(
        mut pc: usize,
        mut fp: usize,
        trampoline_sp: usize,
        mut f: impl FnMut(Frame) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        log::trace!("=== Tracing through contiguous sequence of Wasm frames ===");
        log::trace!("trampoline_sp = 0x{:016x}", trampoline_sp);
        log::trace!("   initial pc = 0x{:016x}", pc);
        log::trace!("   initial fp = 0x{:016x}", fp);

        // We already checked for this case in the `trace_with_trap_state`
        // caller.
        assert_ne!(pc, 0);
        assert_ne!(fp, 0);
        assert_ne!(trampoline_sp, 0);

        arch::assert_entry_sp_is_aligned(trampoline_sp);

        loop {
            // At the start of each iteration of the loop, we know that `fp` is
            // a frame pointer from Wasm code. Therefore, we know it is not
            // being used as an extra general-purpose register, and it is safe
            // dereference to get the PC and the next older frame pointer.

            // The stack grows down, and therefore any frame pointer we are
            // dealing with should be less than the stack pointer on entry
            // to Wasm.
            assert!(trampoline_sp >= fp, "{trampoline_sp:#x} >= {fp:#x}");

            arch::assert_fp_is_aligned(fp);

            log::trace!("--- Tracing through one Wasm frame ---");
            log::trace!("pc = {:p}", pc as *const ());
            log::trace!("fp = {:p}", fp as *const ());

            f(Frame { pc, fp })?;

            pc = arch::get_next_older_pc_from_fp(fp);

            // We rely on this offset being zero for all supported architectures
            // in `crates/cranelift/src/component/compiler.rs` when we set the
            // Wasm exit FP. If this ever changes, we will need to update that
            // code as well!
            assert_eq!(arch::NEXT_OLDER_FP_FROM_FP_OFFSET, 0);

            // Get the next older frame pointer from the current Wasm frame
            // pointer.
            //
            // The next older frame pointer may or may not be a Wasm frame's
            // frame pointer, but it is trusted either way (i.e. is actually a
            // frame pointer and not being used as a general-purpose register)
            // because we always enter Wasm from the host via a trampoline, and
            // this trampoline maintains a proper frame pointer.
            //
            // We want to detect when we've reached the trampoline, and break
            // out of this stack-walking loop. All of our architectures' stacks
            // grow down and look something vaguely like this:
            //
            //     | ...               |
            //     | Native Frames     |
            //     | ...               |
            //     |-------------------|
            //     | ...               | <-- Trampoline FP            |
            //     | Trampoline Frame  |                              |
            //     | ...               | <-- Trampoline SP            |
            //     |-------------------|                            Stack
            //     | Return Address    |                            Grows
            //     | Previous FP       | <-- Wasm FP                Down
            //     | ...               |                              |
            //     | Wasm Frames       |                              |
            //     | ...               |                              V
            //
            // The trampoline records its own stack pointer (`trampoline_sp`),
            // which is guaranteed to be above all Wasm frame pointers but at or
            // below its own frame pointer. It is usually two words above the
            // Wasm frame pointer (at least on x86-64, exact details vary across
            // architectures) but not always: if the first Wasm function called
            // by the host has many arguments, some of them could be passed on
            // the stack in between the return address and the trampoline's
            // frame.
            //
            // To check when we've reached the trampoline frame, it is therefore
            // sufficient to check when the next frame pointer is greater than
            // or equal to `trampoline_sp` (except s390x, where it needs to be
            // strictly greater than).
            let next_older_fp = *(fp as *mut usize).add(arch::NEXT_OLDER_FP_FROM_FP_OFFSET);
            if arch::reached_entry_sp(next_older_fp, trampoline_sp) {
                log::trace!("=== Done tracing contiguous sequence of Wasm frames ===");
                return ControlFlow::Continue(());
            }

            // Because the stack always grows down, the older FP must be greater
            // than the current FP.
            assert!(next_older_fp > fp, "{next_older_fp:#x} > {fp:#x}");
            fp = next_older_fp;
        }
    }

    /// Iterate over the frames inside this backtrace.
    pub fn frames<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a Frame> + DoubleEndedIterator + 'a {
        self.0.iter()
    }
}
