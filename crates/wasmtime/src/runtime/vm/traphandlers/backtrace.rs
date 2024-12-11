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
//! `crates/wasmtime/src/runtime/vm/trampolines`). The most recent entry is
//! stored in `VMRuntimeLimits` and older entries are saved in
//! `CallThreadState`. This lets us identify ranges of contiguous Wasm frames on
//! the stack.
//!
//! To solve (2) and walk the Wasm frames within a region of contiguous Wasm
//! frames on the stack, we configure Cranelift's `preserve_frame_pointers =
//! true` setting. Then we can do simple frame pointer traversal starting at the
//! exit FP and stopping once we reach the entry SP (meaning that the next older
//! frame is a host frame).

use crate::prelude::*;
use crate::runtime::store::StoreOpaque;
use crate::runtime::vm::continuation::stack_chain::StackChain;
use crate::runtime::vm::{
    traphandlers::{tls, CallThreadState},
    Unwind, VMRuntimeLimits,
};
use core::ops::ControlFlow;
use wasmtime_continuations::StackLimits;

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
    pub fn new(store: &StoreOpaque) -> Backtrace {
        let limits = store.runtime_limits();
        let unwind = store.unwinder();
        tls::with(|state| match state {
            Some(state) => unsafe { Self::new_with_trap_state(limits, unwind, state, None) },
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
        unwind: &dyn Unwind,
        state: &CallThreadState,
        trap_pc_and_fp: Option<(usize, usize)>,
    ) -> Backtrace {
        let mut frames = vec![];
        Self::trace_with_trap_state(limits, unwind, state, trap_pc_and_fp, |frame| {
            frames.push(frame);
            ControlFlow::Continue(())
        });
        Backtrace(frames)
    }

    /// Walk the current Wasm stack, calling `f` for each frame we walk.
    pub fn trace(store: &StoreOpaque, f: impl FnMut(Frame) -> ControlFlow<()>) {
        let limits = store.runtime_limits();
        let unwind = store.unwinder();
        tls::with(|state| match state {
            Some(state) => unsafe { Self::trace_with_trap_state(limits, unwind, state, None, f) },
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
        unwind: &dyn Unwind,
        state: &CallThreadState,
        trap_pc_and_fp: Option<(usize, usize)>,
        mut f: impl FnMut(Frame) -> ControlFlow<()>,
    ) {
        if cfg!(feature = "wasmfx_baseline") && cfg!(not(feature = "wasmfx_no_baseline")) {
            if crate::runtime::vm::continuation::baseline::has_ever_run_continuation() {
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
                assert!(core::ptr::eq(limits, state.limits));
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
        let activations = core::iter::once((
            first_wasm_state_stack_chain,
            last_wasm_exit_pc,
            last_wasm_exit_fp,
            *(*limits).last_wasm_entry_fp.get(),
        ))
        .chain(
            first_wasm_state
                .iter()
                .flat_map(|state| state.iter())
                .filter(|state| core::ptr::eq(limits, state.limits))
                .map(|state| {
                    (
                        None,
                        state.old_last_wasm_exit_pc(),
                        state.old_last_wasm_exit_fp(),
                        state.old_last_wasm_entry_fp(),
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

        for (_chain, pc, fp, sp) in activations {
            if cfg!(feature = "wasmfx_baseline") && cfg!(not(feature = "wasmfx_no_baseline")) {
                if let ControlFlow::Break(()) = Self::trace_through_wasm(unwind, pc, fp, sp, &mut f)
                {
                    log::trace!("====== Done Capturing Backtrace (closure break) ======");
                    return;
                }
            } else {
                if let ControlFlow::Break(()) =
                    Self::trace_through_continuations(unwind, _chain, pc, fp, sp, &mut f)
                {
                    log::trace!("====== Done Capturing Backtrace (closure break) ======");
                    return;
                }
            }
        }

        log::trace!("====== Done Capturing Backtrace (reached end of activations) ======");
    }

    #[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))]
    unsafe fn trace_through_continuations(
        _unwind: &dyn Unwind,
        _chain: Option<&StackChain>,
        _pc: usize,
        _fp: usize,
        _trampoline_sp: usize,
        _f: impl FnMut(Frame) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        unimplemented!()
    }

    #[cfg(any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"))]
    unsafe fn trace_through_continuations(
        unwind: &dyn Unwind,
        chain: Option<&StackChain>,
        pc: usize,
        fp: usize,
        trampoline_sp: usize,
        mut f: impl FnMut(Frame) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        use crate::runtime::vm::continuation::imp::VMContRef;

        // Handle the stack that is currently running (which may be a
        // continuation or the main stack).
        Self::trace_through_wasm(unwind, pc, fp, trampoline_sp, &mut f)?;

        chain.map_or(ControlFlow::Continue(()), |chain| {
            debug_assert_ne!(*chain, StackChain::Absent);

            let stack_limits_vec: Vec<*mut StackLimits> =
                chain.clone().into_stack_limits_iter().collect();
            let continuations_vec: Vec<*mut VMContRef> =
                chain.clone().into_continuation_iter().collect();

            // The StackLimits of the currently running stack (whether that's a
            // continuation or the main stack) contains undefined data, the
            // information about that stack is saved in the Store's
            // `VMRuntimeLimits` and handled at the top of this function already.
            // That's why we ignore `stack_limits_vec[0]`.
            //
            // Note that a continuation stack's ControlContext stores
            // information about how to resume execution *in its parent*. Thus,
            // we combine the information from continuations_vec[i] with
            // stack_limits_vec[i + 1] below to get information about a
            // particular stack.
            //
            // There must be exactly one more `StackLimits` object than there
            // are continuations, due to the main stack having one, too.
            assert_eq!(stack_limits_vec.len(), continuations_vec.len() + 1);

            for i in 0..continuations_vec.len() {
                let (continuation, parent_continuation, parent_limits) = unsafe {
                    // The continuation whose control context we want to
                    // access, to get information about how to continue
                    // execution in its parent.
                    let continuation = &*continuations_vec[i];

                    // The stack limits describing the parent of `continuation`.
                    let parent_limits = &*stack_limits_vec[i + 1];

                    // The parent of `continuation`, if the parent is itself a
                    // continuation. Otherwise, if `continuation` is the last
                    // continuation (i.e., its parent is the main stack), this is
                    // None.
                    let parent_continuation = if i + 1 < continuations_vec.len() {
                        Some(&*continuations_vec[i + 1])
                    } else {
                        None
                    };
                    (continuation, parent_continuation, parent_limits)
                };
                let fiber_stack = continuation.fiber_stack();
                let resume_pc = fiber_stack.control_context_instruction_pointer();
                let resume_fp = fiber_stack.control_context_frame_pointer();

                // If the parent is indeed a continuation, we know the
                // boundaries of its stack and can perform some extra checks.
                let parent_stack_range = parent_continuation.and_then(|p| p.fiber_stack().range());
                parent_stack_range.inspect(|parent_stack_range| {
                    debug_assert!(parent_stack_range.contains(&resume_fp));
                    debug_assert!(parent_stack_range.contains(&parent_limits.last_wasm_entry_fp));
                    debug_assert!(parent_stack_range.contains(&parent_limits.stack_limit));
                });

                Self::trace_through_wasm(
                    unwind,
                    resume_pc,
                    resume_fp,
                    parent_limits.last_wasm_entry_fp,
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
        unwind: &dyn Unwind,
        mut pc: usize,
        mut fp: usize,
        trampoline_fp: usize,
        mut f: impl FnMut(Frame) -> ControlFlow<()>,
    ) -> ControlFlow<()> {
        log::trace!("=== Tracing through contiguous sequence of Wasm frames ===");
        log::trace!("trampoline_fp = 0x{:016x}", trampoline_fp);
        log::trace!("   initial pc = 0x{:016x}", pc);
        log::trace!("   initial fp = 0x{:016x}", fp);

        // We already checked for this case in the `trace_with_trap_state`
        // caller.
        assert_ne!(pc, 0);
        assert_ne!(fp, 0);
        assert_ne!(trampoline_fp, 0);

        // This loop will walk the linked list of frame pointers starting at
        // `fp` and going up until `trampoline_fp`. We know that both `fp` and
        // `trampoline_fp` are "trusted values" aka generated and maintained by
        // Cranelift. This means that it should be safe to walk the linked list
        // of pointers and inspect wasm frames.
        //
        // Note, though, that any frames outside of this range are not
        // guaranteed to have valid frame pointers. For example native code
        // might be using the frame pointer as a general purpose register. Thus
        // we need to be careful to only walk frame pointers in this one
        // contiguous linked list.
        //
        // To know when to stop iteration all architectures' stacks currently
        // look something like this:
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
        // The trampoline records its own frame pointer (`trampoline_fp`),
        // which is guaranteed to be above all Wasm. To check when we've
        // reached the trampoline frame, it is therefore sufficient to
        // check when the next frame pointer is equal to `trampoline_fp`. Once
        // that's hit then we know that the entire linked list has been
        // traversed.
        //
        // Note that it might be possible that this loop doesn't execute at all.
        // For example if the entry trampoline called wasm which `return_call`'d
        // an imported function which is an exit trampoline, then
        // `fp == trampoline_fp` on the entry of this function, meaning the loop
        // won't actually execute anything.
        while fp != trampoline_fp {
            // At the start of each iteration of the loop, we know that `fp` is
            // a frame pointer from Wasm code. Therefore, we know it is not
            // being used as an extra general-purpose register, and it is safe
            // dereference to get the PC and the next older frame pointer.
            //
            // The stack also grows down, and therefore any frame pointer we are
            // dealing with should be less than the frame pointer on entry to
            // Wasm. Finally also assert that it's aligned correctly as an
            // additional sanity check.
            assert!(trampoline_fp > fp, "{trampoline_fp:#x} > {fp:#x}");
            unwind.assert_fp_is_aligned(fp);

            log::trace!("--- Tracing through one Wasm frame ---");
            log::trace!("pc = {:p}", pc as *const ());
            log::trace!("fp = {:p}", fp as *const ());

            f(Frame { pc, fp })?;

            pc = unwind.get_next_older_pc_from_fp(fp);

            // We rely on this offset being zero for all supported architectures
            // in `crates/cranelift/src/component/compiler.rs` when we set the
            // Wasm exit FP. If this ever changes, we will need to update that
            // code as well!
            assert_eq!(unwind.next_older_fp_from_fp_offset(), 0);

            // Get the next older frame pointer from the current Wasm frame
            // pointer.
            let next_older_fp = *(fp as *mut usize).add(unwind.next_older_fp_from_fp_offset());

            // Because the stack always grows down, the older FP must be greater
            // than the current FP.
            assert!(next_older_fp > fp, "{next_older_fp:#x} > {fp:#x}");
            fp = next_older_fp;
        }

        log::trace!("=== Done tracing contiguous sequence of Wasm frames ===");
        ControlFlow::Continue(())
    }

    /// Iterate over the frames inside this backtrace.
    pub fn frames<'a>(
        &'a self,
    ) -> impl ExactSizeIterator<Item = &'a Frame> + DoubleEndedIterator + 'a {
        self.0.iter()
    }
}
