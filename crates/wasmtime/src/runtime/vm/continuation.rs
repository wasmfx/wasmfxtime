//! Continuations TODO

cfg_if::cfg_if! {
    if #[cfg(feature = "wasmfx_baseline")] {
        pub use baseline as imp;
    } else {
        pub use optimized as imp;
    }
}

/// A continuation object is a handle to a continuation reference
/// (i.e. an actual stack). A continuation object only be consumed
/// once. The linearity is checked dynamically in the generated code
/// by comparing the revision witness embedded in the pointer to the
/// actual revision counter on the continuation reference.
#[cfg_attr(
    feature = "unsafe_disable_continuation_linearity_check",
    allow(dead_code)
)]
pub mod safe_vm_contobj {
    use super::imp::VMContRef;
    use core::ptr::NonNull;

    // This type is 16 byte aligned so that we can do an aligned load into a
    // 128bit value (see [wasmtime_cranelift::wasmfx::shared::vm_contobj_type]).
    #[repr(C, align(16))]
    #[derive(Debug, Clone, Copy)]
    pub struct VMContObj {
        pub revision: u64,
        pub contref: NonNull<VMContRef>,
    }

    impl VMContObj {
        pub fn new(contref: NonNull<VMContRef>, revision: u64) -> Self {
            Self { contref, revision }
        }
    }
}

/// This version of `VMContObj` does not actually store a revision counter. It is
/// used when we opt out of the linearity check using the
/// `unsafe_disable_continuation_linearity_check` feature
#[cfg_attr(
    not(feature = "unsafe_disable_continuation_linearity_check"),
    allow(dead_code)
)]
pub mod unsafe_vm_contobj {
    use super::imp::VMContRef;
    use core::ptr::NonNull;

    #[repr(transparent)]
    #[derive(Debug, Clone, Copy)]
    pub struct VMContObj(NonNull<VMContRef>);

    impl VMContObj {
        pub fn new(contref: NonNull<VMContRef>, _revision: u64) -> Self {
            Self(contref)
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "unsafe_disable_continuation_linearity_check")] {
        pub use unsafe_vm_contobj::*;
    } else {
        pub use safe_vm_contobj::*;
    }
}

unsafe impl Send for VMContObj {}
unsafe impl Sync for VMContObj {}

#[cfg(not(feature = "wasmfx_baseline"))]
pub mod optimized {
    use super::stack_chain::StackChain;
    use crate::runtime::vm::{
        fibre::Fiber,
        vmcontext::{VMFuncRef, ValRaw},
        Instance, TrapReason,
    };
    use core::cmp;
    use core::mem;
    use wasmtime_continuations::{debug_println, ENABLE_DEBUG_PRINTING};
    pub use wasmtime_continuations::{Payloads, StackLimits, State, SwitchDirection};
    use wasmtime_environ::prelude::*;

    /// Fibers used for continuations
    pub type ContinuationFiber = Fiber;
    pub type FiberStack = crate::runtime::vm::fibre::FiberStack;

    /// TODO
    #[repr(C)]
    pub struct VMContRef {
        /// The limits of this continuation's stack.
        pub limits: StackLimits,

        /// The parent of this continuation, which may be another continuation, the
        /// main stack, or absent (in case of a suspended continuation).
        pub parent_chain: StackChain,

        /// The underlying `Fiber`.
        pub fiber: ContinuationFiber,

        /// Used to store
        /// 1. The arguments to the function passed to cont.new
        /// 2. The return values of that function
        /// Note that this is *not* used for tag payloads.
        pub args: Payloads,

        /// Once a continuation is suspended, this buffer is used to hold payloads
        /// provided by cont.bind and resume and received at the suspend site.
        /// In particular, this may only be Some when `state` is `Invoked`.
        pub tag_return_values: Payloads,

        /// Indicates the state of this continuation.
        pub state: State,

        /// Revision counter.
        pub revision: u64,
    }

    /// TODO
    pub fn cont_ref_forward_tag_return_values_buffer(
        parent: *mut VMContRef,
        child: *mut VMContRef,
    ) -> Result<(), TrapReason> {
        let parent = unsafe {
            parent.as_mut().ok_or_else(|| {
                TrapReason::user_without_backtrace(anyhow::anyhow!(
                    "Attempt to dereference null (parent) VMContRef"
                ))
            })?
        };
        let child = unsafe {
            child.as_mut().ok_or_else(|| {
                TrapReason::user_without_backtrace(anyhow::anyhow!(
                    "Attempt to dereference null (child) VMContRef"
                ))
            })?
        };
        assert!(parent.state == State::Invoked);
        assert!(child.state == State::Invoked);
        assert!(child.tag_return_values.length == 0);

        mem::swap(&mut child.tag_return_values, &mut parent.tag_return_values);
        Ok(())
    }

    /// TODO
    #[inline(always)]
    pub fn drop_cont_ref(instance: &mut Instance, contref: *mut VMContRef) {
        // Note that continuation references do not own their parents, hence we ignore
        // parent fields here.

        let contref: Box<VMContRef> = unsafe { Box::from_raw(contref) };
        instance.wasmfx_deallocate_stack(contref.fiber.stack());
        if contref.args.data.is_null() {
            debug_assert!(contref.args.length as usize == 0);
            debug_assert!(contref.args.capacity as usize == 0);
        } else {
            unsafe {
                let _: Vec<u128> = Vec::from_raw_parts(
                    contref.args.data,
                    contref.args.length as usize,
                    contref.args.capacity as usize,
                );
            };
        }
        let payloads = &contref.tag_return_values;
        if payloads.data.is_null() {
            debug_assert!(payloads.length as usize == 0);
            debug_assert!(payloads.capacity as usize == 0);
        } else {
            let _: Vec<u128> = unsafe {
                Vec::from_raw_parts(
                    payloads.data,
                    payloads.length as usize,
                    payloads.capacity as usize,
                )
            };
        }
    }

    /// TODO
    #[inline(always)]
    pub fn cont_new(
        instance: &mut Instance,
        func: *mut u8,
        param_count: u32,
        result_count: u32,
    ) -> Result<*mut VMContRef, TrapReason> {
        let caller_vmctx = instance.vmctx();

        let capacity = cmp::max(param_count, result_count);
        let payload = Payloads::new(capacity);

        let wasmfx_config = unsafe { &*(*instance.store()).wasmfx_config() };
        // TODO(frank-emrich) Currently, the general `stack_limit` configuration
        // option of wasmtime is unrelated to the stack size of our fiber stack.
        let stack_size = wasmfx_config.stack_size;
        let red_zone_size = wasmfx_config.red_zone_size;

        let fiber = {
            let stack = instance.wasmfx_allocate_stack().map_err(|_error| {
                TrapReason::user_without_backtrace(anyhow::anyhow!(
                    "Fiber stack allocation failed!"
                ))
            })?;
            Fiber::new(
                stack,
                func.cast::<VMFuncRef>(),
                caller_vmctx,
                payload.data as *mut ValRaw,
                payload.capacity as usize,
            )
            .map_err(|_error| {
                TrapReason::user_without_backtrace(anyhow::anyhow!("Fiber construction failed!"))
            })?
        };

        let tsp = fiber.stack().top().unwrap();
        let stack_limit = unsafe { tsp.sub(stack_size - red_zone_size) } as usize;
        let contref = Box::new(VMContRef {
            revision: 0,
            limits: StackLimits::with_stack_limit(stack_limit),
            fiber,
            parent_chain: StackChain::Absent,
            args: payload,
            tag_return_values: Payloads::new(0),
            state: State::Allocated,
        });

        // TODO(dhil): we need memory clean up of
        // continuation reference objects.
        let pointer = Box::into_raw(contref);
        debug_println!("Created contref @ {:p}", pointer);
        Ok(pointer)
    }

    /// TODO
    #[inline(always)]
    pub fn resume(
        instance: &mut Instance,
        contref: *mut VMContRef,
        parent_stack_limits: *mut StackLimits,
    ) -> Result<SwitchDirection, TrapReason> {
        let cont = unsafe {
            contref.as_ref().ok_or_else(|| {
                TrapReason::user_without_backtrace(anyhow::anyhow!(
                    "Attempt to dereference null VMContRef!"
                ))
            })?
        };
        assert!(cont.state == State::Allocated || cont.state == State::Invoked);

        if ENABLE_DEBUG_PRINTING {
            let chain = instance.typed_continuations_stack_chain();
            // SAFETY: We maintain as an invariant that the stack chain field in the
            // VMContext is non-null and contains a chain of zero or more
            // StackChain::Continuation values followed by StackChain::Main.
            match unsafe { (**chain).0.get_mut() } {
                StackChain::Continuation(running_contref) => {
                    debug_assert_eq!(contref, *running_contref);
                    debug_println!(
                        "Resuming contref @ {:p}, previously running contref is {:p}",
                        contref,
                        running_contref
                    )
                }
                _ => {
                    // Before calling this function as a libcall, we must have set
                    // the parent of the to-be-resumed continuation to the
                    // previously running one. Hence, we must see a
                    // `StackChain::Continuation` variant.
                    return Err(TrapReason::user_without_backtrace(anyhow::anyhow!(
                        "Invalid StackChain value in VMContext"
                    )));
                }
            }
        }

        // See the comment on `wasmtime_continuations::StackChain` for a description
        // of the invariants that we maintain for the various stack limits.
        unsafe {
            let runtime_limits = &**instance.runtime_limits();

            (*parent_stack_limits).stack_limit = *runtime_limits.stack_limit.get();
            (*parent_stack_limits).last_wasm_entry_sp = *runtime_limits.last_wasm_entry_sp.get();
            // These last two values were only just updated in the `runtime_limits`
            // because we entered the current libcall.
            (*parent_stack_limits).last_wasm_exit_fp = *runtime_limits.last_wasm_exit_fp.get();
            (*parent_stack_limits).last_wasm_exit_pc = *runtime_limits.last_wasm_exit_pc.get();

            *runtime_limits.stack_limit.get() = (*contref).limits.stack_limit;
            *runtime_limits.last_wasm_entry_sp.get() = (*contref).limits.last_wasm_entry_sp;
        }

        Ok(cont.fiber.resume())
    }

    /// TODO
    #[inline(always)]
    pub fn suspend(instance: &mut Instance, tag_index: u32) -> Result<(), TrapReason> {
        let chain_ptr = instance.typed_continuations_stack_chain();

        // TODO(dhil): This should be handled in generated code.
        // SAFETY: We maintain as an invariant that the stack chain field in the
        // VMContext is non-null and contains a chain of zero or more
        // StackChain::Continuation values followed by StackChain::Main.
        let chain = unsafe { (**chain_ptr).0.get_mut() };
        let running = match chain {
            StackChain::Absent => Err(TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Internal error: StackChain not initialised"
            ))),
            StackChain::MainStack { .. } => Err(TrapReason::user_without_backtrace(
                anyhow::anyhow!("Calling suspend outside of a continuation"),
            )),
            StackChain::Continuation(running) => {
                // SAFETY: See above.
                Ok(unsafe { &**running })
            }
        }?;

        let fiber = &running.fiber;

        let stack_ptr = fiber.stack().top().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Failed to retrieve stack top pointer!"
            ))
        })?;
        debug_println!(
            "Suspending while running {:p}, parent is {:?}",
            running,
            running.parent_chain
        );

        let suspend = crate::runtime::vm::fibre::unix::Suspend::from_top_ptr(stack_ptr);
        let payload = SwitchDirection::suspend(tag_index);
        Ok(suspend.switch(payload))
    }

    // Tests
    #[test]
    fn offset_and_size_constants() {
        use memoffset;
        use wasmtime_continuations::offsets::*;

        assert_eq!(
            memoffset::offset_of!(VMContRef, limits),
            vm_cont_ref::LIMITS
        );
        assert_eq!(
            memoffset::offset_of!(VMContRef, parent_chain),
            vm_cont_ref::PARENT_CHAIN
        );
        assert_eq!(memoffset::offset_of!(VMContRef, fiber), vm_cont_ref::FIBER);
        assert_eq!(memoffset::offset_of!(VMContRef, args), vm_cont_ref::ARGS);
        assert_eq!(
            memoffset::offset_of!(VMContRef, tag_return_values),
            vm_cont_ref::TAG_RETURN_VALUES
        );
        assert_eq!(memoffset::offset_of!(VMContRef, state), vm_cont_ref::STATE);

        assert_eq!(
            std::mem::size_of::<ContinuationFiber>(),
            CONTINUATION_FIBER_SIZE
        );
        assert_eq!(core::mem::size_of::<StackChain>(), STACK_CHAIN_SIZE);

        assert_eq!(
            memoffset::offset_of!(VMContRef, revision),
            vm_cont_ref::REVISION
        );
    }
}

#[cfg(feature = "wasmfx_baseline")]
pub mod baseline {
    use super::stack_chain::{StackChain, StackLimits};
    use crate::runtime::vm::{Instance, TrapReason, VMFuncRef, VMOpaqueContext, ValRaw};
    use core::{cell::Cell, cell::RefCell, cmp, mem};
    use wasmtime_environ::prelude::*;
    use wasmtime_fiber::{Fiber, Suspend};

    type ContinuationFiber = Fiber<'static, &'static mut Instance, u32, ()>;
    pub type FiberStack = wasmtime_fiber::FiberStack;
    type Yield = Suspend<&'static mut Instance, u32, ()>;

    /// The baseline VM continuation record.
    ///
    /// It is a linked list of continuation records. Each element in
    /// the list consists of a pointer to an actual
    /// wasmtime_fiber::Fiber, a suspend object, a parent pointer, an
    /// arguments buffer, and a return buffer.
    #[repr(C)]
    pub struct VMContRef {
        /// Revision counter.
        pub revision: u64,
        pub fiber: Box<ContinuationFiber>,
        pub suspend: *mut Yield,
        pub limits: StackLimits,
        pub parent_chain: StackChain,
        pub parent: *mut VMContRef,
        pub args: Vec<u128>,
        pub values: Vec<u128>,
        pub _marker: core::marker::PhantomPinned,
    }

    // We use thread local state to simulate the VMContext. The use of
    // thread local state is necessary to reliably pass the testsuite,
    // as the test driver is multi-threaded.
    thread_local! {
        // The current continuation, i.e. the currently executing
        // continuation.
        static CC: Cell<*mut VMContRef> = Cell::new(core::ptr::null_mut());
        // A buffer to help propagate tag payloads across
        // continuations.
        static SUSPEND_PAYLOADS: RefCell<Vec<u128>> = RefCell::new(vec![]);

        // This acts like a fuse that is set to true if this thread has ever
        // executed a continuation (e.g., run `resume`).
        static HAS_EVER_RUN_CONTINUATION: Cell<bool> = Cell::new(false);
    }

    /// Allocates a new continuation in suspended mode.
    #[inline(always)]
    pub fn cont_new(
        instance: &mut Instance,
        func: *mut u8,
        param_count: usize,
        result_count: usize,
    ) -> Result<*mut VMContRef, TrapReason> {
        let capacity = cmp::max(param_count, result_count);
        let mut values: Vec<u128> = Vec::with_capacity(capacity);

        let fiber = {
            let stack = instance
                .wasmfx_allocate_stack()
                .map_err(|error| TrapReason::user_without_backtrace(error.into()))?;
            let fiber = match unsafe { func.cast::<VMFuncRef>().as_ref() } {
                None => Fiber::new(stack, |_instance: &mut Instance, _suspend: &mut Yield| {
                    panic!("Attempt to invoke null VMFuncRef!");
                }),
                Some(func_ref) => {
                    let callee_ctx = func_ref.vmctx;

                    let vals_ptr = values.as_mut_ptr();
                    Fiber::new(
                        stack,
                        move |instance: &mut Instance, suspend: &mut Yield| unsafe {
                            let caller_ctx = VMOpaqueContext::from_vmcontext(instance.vmctx());
                            // NOTE(dhil): The cast `suspend as *mut Yield`
                            // side-steps the need for mentioning the lifetime
                            // of `Yield`. In this case it is safe, because
                            // Yield lives as long as the object it is
                            // embedded in.
                            (*get_current_continuation()).suspend = suspend as *mut Yield;
                            let results = (func_ref.array_call)(
                                callee_ctx,
                                caller_ctx,
                                vals_ptr.cast::<ValRaw>(),
                                capacity,
                            );
                            // As a precaution we null the suspender.
                            (*get_current_continuation()).suspend = core::ptr::null_mut();
                            return results;
                        },
                    )
                }
            };
            Box::new(fiber.map_err(|error| TrapReason::user_without_backtrace(error.into()))?)
        };

        let contref = Box::new(VMContRef {
            revision: 0,
            limits: StackLimits::with_stack_limit(0),
            parent_chain: StackChain::Absent,
            parent: core::ptr::null_mut(),
            suspend: core::ptr::null_mut(),
            fiber,
            args: Vec::with_capacity(param_count),
            values,
            _marker: core::marker::PhantomPinned,
        });

        // TODO(dhil): we need memory clean up of
        // continuation reference objects.
        debug_assert!(!contref.fiber.stack().top().unwrap().is_null());
        Ok(Box::into_raw(contref))
    }

    /// Continues a given continuation.
    #[inline(always)]
    pub fn resume(instance: &mut Instance, contref: &mut VMContRef) -> Result<u32, TrapReason> {
        // Trigger fuse
        if !HAS_EVER_RUN_CONTINUATION.get() {
            HAS_EVER_RUN_CONTINUATION.set(true);
        }

        // Attach parent.
        debug_assert!(contref.parent.is_null());
        contref.parent = get_current_continuation();
        // Append arguments to the function args/return buffer if this
        // is the initial resume. Note: the `contref.args` buffer is
        // appended in the generated code.
        //
        // NOTE(dhil): The `suspend` field is set during the initial
        // invocation.
        if contref.suspend.is_null() {
            debug_assert!(contref.values.len() == 0);
            debug_assert!(contref.args.len() <= contref.values.capacity());
            contref.values.append(&mut contref.args);
            contref.args.clear();
        }
        // Change the current continuation.
        set_current_continuation(contref);
        unsafe {
            (*(*(*instance.store()).vmruntime_limits())
                .stack_limit
                .get_mut()) = 0
        };

        // Resume the current continuation.
        contref
            .fiber
            .resume(instance)
            .map(move |()| {
                // This lambda is run whenever the continuation ran to
                // completion. In this case we update the current
                // continuation to bet the parent of this
                // continuation.
                set_current_continuation(contref.parent);
                // The value zero signals control returned normally.
                return 0;
            })
            .or_else(|tag| {
                // This lambda is run whenever a suspension occurred
                // inside the continuation. In this case we set the
                // high bit of the return value to signal control
                // returned via a suspend.
                let signal_mask = 0xf000_0000;
                debug_assert_eq!(tag & signal_mask, 0);
                return Ok(tag | signal_mask);
            })
    }

    /// Suspends a the current continuation.
    #[inline(always)]
    pub fn suspend(_instance: &mut Instance, tag_index: u32) -> Result<(), TrapReason> {
        let cc = get_current_continuation();
        if cc.is_null() {
            let trap = TrapReason::Wasm(wasmtime_environ::Trap::UnhandledTag);
            return Err(trap);
        }
        let contref = unsafe { cc.as_mut().unwrap() };
        let parent = mem::replace(&mut contref.parent, core::ptr::null_mut());
        set_current_continuation(parent);
        unsafe { contref.suspend.as_mut().unwrap().suspend(tag_index) };
        Ok(())
    }

    /// Forwards handling from the current continuation to its parent.
    #[inline(always)]
    pub fn forward(
        instance: &mut Instance,
        tag_index: u32,
        subcont: &mut VMContRef,
    ) -> Result<(), TrapReason> {
        let cc = get_current_continuation();
        suspend(instance, tag_index)?;
        debug_assert!(get_current_continuation() == cc);
        move_continuation_arguments(unsafe { cc.as_mut().unwrap() }, subcont);
        Ok(())
    }

    /// Deallocates a gives continuation reference.
    #[inline(always)]
    pub fn drop_continuation_reference(instance: &mut Instance, contref: *mut VMContRef) {
        // Note that continuation objects do not own their parents, so
        // we let the parent object leak.
        let contref: Box<VMContRef> = unsafe { Box::from_raw(contref) };
        instance.wasmfx_deallocate_stack(contref.fiber.stack());
        let _: Box<ContinuationFiber> = contref.fiber;
        let _: Vec<u128> = contref.args;
        let _: Vec<u128> = contref.values;
    }

    /// Clears the argument buffer on a given continuation reference.
    #[inline(always)]
    pub fn clear_arguments(_instance: &mut Instance, contref: &mut VMContRef) {
        contref.args.clear();
    }

    /// Returns the pointer to the argument buffer of a given
    /// continuation reference.
    #[inline(always)]
    pub fn get_arguments_ptr(
        _instance: &mut Instance,
        contref: &mut VMContRef,
        nargs: usize,
    ) -> *mut u128 {
        let mut offset: isize = 0;
        // Zero initialise `nargs` cells for writing.
        if nargs > 0 {
            for _ in 0..nargs {
                contref.args.push(0); // zero initialise
            }
            offset = (contref.args.len() - nargs) as isize;
        }
        unsafe { contref.args.as_mut_ptr().offset(offset) }
    }

    /// Returns the pointer to the (return) values buffer of a given
    /// continuation reference.
    #[inline(always)]
    pub fn get_values_ptr(_instance: &mut Instance, contref: &mut VMContRef) -> *mut u128 {
        contref.values.as_mut_ptr()
    }

    /// Returns the pointer to the tag payloads buffer.
    #[inline(always)]
    pub fn get_payloads_ptr(_instance: &mut Instance, nargs: usize) -> *mut u128 {
        // If `nargs > 0` then we zero-initialise `nargs` cells for
        // writing.
        SUSPEND_PAYLOADS.with(|cell| {
            let mut payloads = cell.borrow_mut();
            if nargs > 0 {
                debug_assert!(payloads.len() == 0);
                for _ in 0..nargs {
                    payloads.push(0); // zero initialise
                }
                debug_assert!(payloads.len() == nargs);
            }
            return payloads.as_mut_ptr();
        })
    }

    /// Clears the tag payloads buffer.
    #[inline(always)]
    pub fn clear_payloads(_instance: &mut Instance) {
        SUSPEND_PAYLOADS.with(|cell| {
            let mut payloads = cell.borrow_mut();
            payloads.clear();
            debug_assert!(payloads.len() == 0)
        })
    }

    /// Moves the arguments of `src` continuation to `dst`
    /// continuation.
    #[inline(always)]
    fn move_continuation_arguments(src: &mut VMContRef, dst: &mut VMContRef) {
        let srclen = src.args.len();
        debug_assert!(dst.args.len() == 0);
        dst.args.append(&mut src.args);
        debug_assert!(src.args.len() == 0);
        debug_assert!(dst.args.len() == srclen);
    }

    /// Gets the current continuation.
    #[inline(always)]
    pub fn get_current_continuation() -> *mut VMContRef {
        CC.get()
    }

    /// Sets the current continuation.
    #[inline(always)]
    fn set_current_continuation(cont: *mut VMContRef) {
        CC.set(cont)
    }

    pub fn has_ever_run_continuation() -> bool {
        HAS_EVER_RUN_CONTINUATION.get()
    }
}

//
// Stack chain
//
pub mod stack_chain {
    use super::imp::VMContRef;
    use core::cell::UnsafeCell;
    pub use wasmtime_continuations::StackLimits;

    /// This type represents a linked lists of stacks, additionally associating a
    /// `StackLimits` object with each element of the list. Here, a "stack" is
    /// either a continuation or the main stack. Note that the linked list character
    /// arises from the fact that `StackChain::Continuation` variants have a pointer
    /// to have `VMContRef`, which in turn has a `parent_chain` value of
    /// type `StackChain`.
    ///
    /// There are generally two uses of such chains:
    ///
    /// 1. The `typed_continuations_chain` field in the VMContext contains such a
    /// chain of stacks, where the head of the list denotes the stack that is
    /// currently executing (either a continuation or the main stack), as well as
    /// the parent stacks, in case of a continuation currently running. Note that in
    /// this case, the linked list must contains 0 or more `Continuation` elements,
    /// followed by a final `MainStack` element. In particular, this list always
    /// ends with `MainStack` and never contains an `Absent` variant.
    ///
    /// 2. When a continuation is suspended, its chain of parents eventually ends
    /// with an `Absent` variant in its `parent_chain` field. Note that a suspended
    /// continuation never appears in the stack chain in the VMContext!
    ///
    ///
    /// As mentioned before, each stack in a `StackChain` has a corresponding
    /// `StackLimits` object. For continuations, this is stored in the `limits`
    /// fields of the corresponding `VMContRef`. For the main stack, the
    /// `MainStack` variant contains a pointer to the
    /// `typed_continuations_main_stack_limits` field of the VMContext.
    ///
    /// The following invariants hold for these `StackLimits` objects, and the data
    /// in `VMRuntimeLimits`.
    ///
    /// Currently executing stack:
    /// For the currently executing stack (i.e., the stack that is at the head of
    /// the VMContext's `typed_continuations_chain` list), the associated
    /// `StackLimits` object contains stale/undefined data. Instead, the live data
    /// describing the limits for the currently executing stack is always maintained
    /// in `VMRuntimeLimits`. Note that as a general rule independently from any
    /// execution of continuations, the `last_wasm_exit*` fields in the
    /// `VMRuntimeLimits` contain undefined values while executing wasm.
    ///
    /// Parents of currently executing stack:
    /// For stacks that appear in the tail of the VMContext's
    /// `typed_continuations_chain` list (i.e., stacks that are not currently
    /// executing themselves, but are a parent of the currently executing stack), we
    /// have the following: All the fields in the stack's StackLimits are valid,
    /// describing the stack's stack limit, and pointers where executing for that
    /// stack entered and exited WASM.
    ///
    /// Suspended continuations:
    /// For suspended continuations (including their parents), we have the
    /// following. Note that the main stack can never be in this state. The
    /// `stack_limit` and `last_enter_wasm_sp` fields of the corresponding
    /// `StackLimits` object contain valid data, while the `last_exit_wasm_*` fields
    /// contain arbitrary values.
    /// There is only one exception to this: Note that a continuation that has been
    /// created with cont.new, but never been resumed so far, is considered
    /// "suspended". However, its `last_enter_wasm_sp` field contains undefined
    /// data. This is justified, because when resume-ing a continuation for the
    /// first time, a native-to-wasm trampoline is called, which sets up the
    /// `last_wasm_entry_sp` in the `VMRuntimeLimits` with the correct value, thus
    /// restoring the necessary invariant.
    #[derive(Debug, Clone, PartialEq)]
    #[repr(usize, C)]
    pub enum StackChain {
        /// If stored in the VMContext, used to indicate that the MainStack entry
        /// has not been set, yet. If stored in a VMContRef's parent_chain
        /// field, means that there is currently no parent.
        Absent = wasmtime_continuations::STACK_CHAIN_ABSENT_DISCRIMINANT,
        /// Represents the main stack.
        MainStack(*mut StackLimits) = wasmtime_continuations::STACK_CHAIN_MAIN_STACK_DISCRIMINANT,
        /// Represents a continuation's stack.
        Continuation(*mut VMContRef) =
            wasmtime_continuations::STACK_CHAIN_CONTINUATION_DISCRIMINANT,
    }

    impl StackChain {
        /// Indicates if `self` is a `MainStack` variant.
        pub fn is_main_stack(&self) -> bool {
            matches!(self, StackChain::MainStack(_))
        }

        /// Returns an iterator over the stacks in this chain.
        /// We don't implement `IntoIterator` because our iterator is unsafe, so at
        /// least this gives us some way of indicating this, even though the actual
        /// unsafety lies in the `next` function.
        ///
        /// # Safety
        ///
        /// This function is not unsafe per see, but it returns an object
        /// whose usage is unsafe.
        pub unsafe fn into_iter(self) -> ContinuationChainIterator {
            ContinuationChainIterator(self)
        }
    }

    /// Iterator for stacks in a stack chain.
    /// Each stack is represented by a tuple `(co_opt, sl)`, where sl is a pointer
    /// to the stack's `StackLimits` object and `co_opt` is a pointer to the
    /// corresponding `VMContRef`, or None for the main stack.
    pub struct ContinuationChainIterator(StackChain);

    impl Iterator for ContinuationChainIterator {
        type Item = (Option<*mut VMContRef>, *mut StackLimits);

        fn next(&mut self) -> Option<Self::Item> {
            match self.0 {
                StackChain::Absent => None,
                StackChain::MainStack(ms) => {
                    let next = (None, ms);
                    self.0 = StackChain::Absent;
                    Some(next)
                }
                StackChain::Continuation(ptr) => {
                    let continuation = unsafe { ptr.as_mut().unwrap() };
                    let next = (Some(ptr), (&mut continuation.limits) as *mut StackLimits);
                    self.0 = continuation.parent_chain.clone();
                    Some(next)
                }
            }
        }
    }

    #[repr(transparent)]
    /// Wraps a `StackChain` in an `UnsafeCell`, in order to store it in a
    /// `StoreOpaque`.
    pub struct StackChainCell(pub UnsafeCell<StackChain>);

    impl StackChainCell {
        /// Indicates if the underlying `StackChain` object has value `Absent`.
        pub fn absent() -> Self {
            StackChainCell(UnsafeCell::new(StackChain::Absent))
        }
    }

    // Since `StackChainCell` objects appear in the `StoreOpaque`,
    // they need to be `Send` and `Sync`.
    // This is safe for the same reason it is for `VMRuntimeLimits` (see comment
    // there): Both types are pod-type with no destructor, and we don't access any
    // of their fields from other threads.
    unsafe impl Send for StackChainCell {}
    unsafe impl Sync for StackChainCell {}
}

//
// Dummy implementations
//

#[allow(missing_docs)]
#[cfg(feature = "wasmfx_baseline")]
pub mod optimized {
    use crate::runtime::vm::{Instance, TrapReason};
    pub use wasmtime_continuations::{StackLimits, SwitchDirection};

    pub type VMContRef = super::baseline::VMContRef;

    pub fn cont_ref_forward_tag_return_values_buffer(
        _parent: *mut VMContRef,
        _child: *mut VMContRef,
    ) -> Result<(), TrapReason> {
        panic!("attempt to execute continuation::optimized::cont_ref_forward_tag_return_values_buffer with `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    pub fn drop_cont_ref(_instance: &mut Instance, _contref: *mut VMContRef) {
        panic!("attempt to execute continuation::optimized::drop_cont_ref with `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    pub fn cont_new(
        _instance: &mut Instance,
        _func: *mut u8,
        _param_count: u32,
        _result_count: u32,
    ) -> Result<*mut VMContRef, TrapReason> {
        panic!("attempt to execute continuation::optimized::cont_new with `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    pub fn resume(
        _instance: &mut Instance,
        _contref: *mut VMContRef,
        _parent_stack_limits: *mut StackLimits,
    ) -> Result<SwitchDirection, TrapReason> {
        panic!("attempt to execute continuation::optimized::resume with `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    pub fn suspend(_instance: &mut Instance, _tag_index: u32) -> Result<(), TrapReason> {
        panic!("attempt to execute continuation::optimized::suspend with `typed_continuation_baseline_implementation` toggled!")
    }
}

#[allow(missing_docs)]
#[cfg(not(feature = "wasmfx_baseline"))]
pub mod baseline {
    use crate::runtime::vm::{Instance, TrapReason};

    #[allow(missing_docs)]
    #[repr(C)]
    pub struct VMContRef();

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn cont_new(
        _instance: &mut Instance,
        _func: *mut u8,
        _param_count: usize,
        _result_count: usize,
    ) -> Result<*mut VMContRef, TrapReason> {
        panic!("attempt to execute continuation::baseline::cont_new without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn resume(_instance: &mut Instance, _contref: &mut VMContRef) -> Result<u32, TrapReason> {
        panic!("attempt to execute continuation::baseline::resume without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn suspend(_instance: &mut Instance, _tag_index: u32) -> Result<(), TrapReason> {
        panic!("attempt to execute continuation::baseline::suspend without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn forward(
        _instance: &mut Instance,
        _tag_index: u32,
        _subcont: &mut VMContRef,
    ) -> Result<(), TrapReason> {
        panic!("attempt to execute continuation::baseline::forward without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn drop_continuation_reference(_instance: &mut Instance, _cont: *mut VMContRef) {
        panic!("attempt to execute continuation::baseline::drop_continuation_reference without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn get_arguments_ptr(
        _instance: &mut Instance,
        _contref: &mut VMContRef,
        _nargs: usize,
    ) -> *mut u8 {
        panic!("attempt to execute continuation::baseline::get_arguments_ptr without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn get_values_ptr(_instance: &mut Instance, _contref: &mut VMContRef) -> *mut u8 {
        panic!("attempt to execute continuation::baseline::get_values_ptr without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn clear_arguments(_instance: &mut Instance, _contref: &mut VMContRef) {
        panic!("attempt to execute continuation::baseline::clear_arguments without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn get_payloads_ptr(_instance: &mut Instance, _nargs: usize) -> *mut u128 {
        panic!("attempt to execute continuation::baseline::get_payloads_ptr without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn clear_payloads(_instance: &mut Instance) {
        panic!("attempt to execute continuation::baseline::clear_payloads without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn get_current_continuation() -> *mut VMContRef {
        panic!("attempt to execute continuation::baseline::get_current_continuation without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn has_ever_run_continuation() -> bool {
        panic!("attempt to execute continuation::baseline::has_ever_run_continuation without `typed_continuation_baseline_implementation` toggled!")
    }
}

mod test {
    #[test]
    fn null_pointer_optimization() {
        // The Rust spec does not technically guarantee that the null pointer
        // optimization applies to a struct containing a NonNull.
        assert_eq!(
            std::mem::size_of::<Option<super::safe_vm_contobj::VMContObj>>(),
            std::mem::size_of::<super::safe_vm_contobj::VMContObj>()
        );
    }
}
