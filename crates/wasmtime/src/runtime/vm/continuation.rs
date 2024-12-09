//! Continuations TODO

cfg_if::cfg_if! {
    if #[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))] {
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
///
/// In the optimized implementation, the continuation logically
/// represented by a VMContObj not only encompasses the pointed-to
/// VMContRef, but also all of its parents:
///
/// ```text
///
///                     +----------------+
///                 +-->|   VMContRef    |
///                 |   +----------------+
///                 |            ^
///                 |            | parent
///                 |            |
///                 |   +----------------+
///                 |   |   VMContRef    |
///                 |   +----------------+
///                 |            ^
///                 |            | parent
///  last ancestor  |            |
///                 |   +----------------+
///                 +---|   VMContRef    |    <--  VMContObj
///                     +----------------+
/// ```
///
/// For performance reasons, the VMContRef at the bottom of this chain
/// (i.e., the one pointed to by the VMContObj) has a pointer to the
/// other end of the chain (i.e., its last ancestor).
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

pub use safe_vm_contobj::*;

unsafe impl Send for VMContObj {}
unsafe impl Sync for VMContObj {}

#[cfg(any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"))]
pub mod optimized {
    use super::stack_chain::StackChain;
    use crate::runtime::vm::{
        vmcontext::{VMFuncRef, ValRaw},
        Instance, TrapReason, VMStore,
    };
    use core::cmp;
    use std::marker::PhantomPinned;
    use wasmtime_continuations::HandlerList;
    #[allow(unused)]
    use wasmtime_continuations::{debug_println, CommonStackInformation, ENABLE_DEBUG_PRINTING};
    pub use wasmtime_continuations::{Payloads, StackLimits, State};

    /// Fibers used for continuations
    pub type FiberStack = crate::runtime::vm::fibre::FiberStack;

    /// TODO
    #[repr(C)]
    pub struct VMContRef {
        /// The `CommonStackInformation` of this continuation's stack.
        pub common_stack_information: CommonStackInformation,

        /// The parent of this continuation, which may be another continuation, the
        /// main stack, or absent (in case of a suspended continuation).
        pub parent_chain: StackChain,

        /// Only used if `common_stack_information.state` is `Suspended` or `Fresh`. In
        /// that case, this points to the end of the stack chain (i.e., the
        /// continuation in the parent chain whose own `parent_chain` field is
        /// `StackChain::Absent`).
        /// Note that this may be a pointer to iself (if the state is `Fresh`, this is always the case).
        pub last_ancestor: *mut VMContRef,

        /// The underlying stack.
        pub stack: FiberStack,

        /// Used to store
        /// 1. The arguments to the function passed to cont.new
        /// 2. The return values of that function
        /// Note that this is *not* used for tag payloads.
        pub args: Payloads,

        /// Once a continuation has been suspended (using suspend or switch),
        /// this buffer is used to hold payloads provided by cont.bind, resume,
        /// and switch. They are received at the suspend site (i.e., the
        /// corrsponding suspend or switch instruction). In particular, this may
        /// not be used while the continuation's state is `Fresh`.
        pub values: Payloads,

        /// Revision counter.
        pub revision: u64,

        /// Tell the compiler that this structure has potential self-references
        /// through the `last_ancestor` pointer.
        _marker: core::marker::PhantomPinned,
    }

    impl VMContRef {
        pub fn fiber_stack(&self) -> &FiberStack {
            &self.stack
        }

        pub fn detach_stack(&mut self) -> FiberStack {
            core::mem::replace(&mut self.stack, FiberStack::unallocated())
        }

        /// This is effectively a `Default` implementation, without calling it
        /// so. Used to create `VMContRef`s when initializing pooling allocator.
        #[allow(clippy::cast_possible_truncation)]
        pub fn empty() -> Self {
            let limits = StackLimits::with_stack_limit(Default::default());
            let state = State::Fresh;
            let handlers =
                HandlerList::new(wasmtime_continuations::INITIAL_HANDLER_LIST_CAPACITY as u32);
            let common_stack_information = CommonStackInformation {
                limits,
                state,
                handlers,
                first_switch_handler_index: 0,
            };
            let parent_chain = StackChain::Absent;
            let last_ancestor = std::ptr::null_mut();
            let stack = FiberStack::unallocated();
            let args = Payloads::new(0);
            let values = Payloads::new(0);
            let revision = 0;
            let _marker = PhantomPinned;

            Self {
                common_stack_information,
                parent_chain,
                last_ancestor,
                stack,
                args,
                values,
                revision,
                _marker,
            }
        }
    }

    impl Drop for VMContRef {
        fn drop(&mut self) {
            // Note that continuation references do not own their parents, and we
            // don't drop them here.

            // `Payloads` must be deallocated explicitly, they are considered non-owning.
            self.args.deallocate();
            self.values.deallocate();

            self.common_stack_information.handlers.deallocate();

            // We would like to enforce the invariant that any continuation that
            // was created for a cont.new (rather than, say, just living in a
            // pool and never being touched), either ran to completion or was
            // cancelled. But failing to do so should yield a custom error,
            // instead of panicking here.
        }
    }

    // These are required so the WasmFX pooling allocator can store a Vec of
    // `VMContRef`s.
    unsafe impl Send for VMContRef {}
    unsafe impl Sync for VMContRef {}

    /// Drops the given continuation, which either means deallocating it
    /// (together with its stack) or returning it (and its stack) to a pool for
    /// later reuse.
    ///
    /// If pooling is enabled, then all `VMContObj`s pointing to this
    /// `VMContRef` must have outdated revision counters. The pool guarantees
    /// that the revision counter stays unchanged if this `VMContRef` is reused.
    ///
    /// If pooling is disabled, then there must be no `VMContObj`s pointing to
    /// this `VMContRef` anymore.
    /// FIXME(frank-emrich) This second condition can currently be violated: We
    /// call this immediately once a continuation returns, at which point such
    /// `VMContObj`s may exist.
    #[inline(always)]
    pub fn drop_cont_ref(instance: &mut Instance, contref: *mut VMContRef) {
        {
            let contref = unsafe { contref.as_mut().unwrap() };
            // A continuation must have run to completion before dropping it.
            debug_assert!(contref.common_stack_information.state == State::Returned);

            // Note that we *could* deallocate the `Payloads` (i.e., `args` and
            // `values`) here, but choose not to:
            // - If we are using on-demand allocation of `VMContRef`s, the
            //   `Payloads` get deallocated as part of `Drop`-ing the `VMContRef`.
            // - If we are using the pooling allocator, we deliberately return
            //   the `contref` to the pool with its `Payloads` still allocated.
            //   When the `contref` is handed out subsequently on another
            //   allocation requesdt, we can resize the `Payloads` if needed.
            //
            // So instead we just clear the elements.
            contref.args.clear();
            contref.values.clear();
        }

        // The WasmFX allocator decides if "deallocating" a continuation means
        // putting it back into the pool or actually deallocating it.
        instance.wasmfx_deallocate_continuation(contref);
    }

    /// TODO
    #[inline(always)]
    pub fn cont_new(
        store: &mut dyn VMStore,
        instance: &mut Instance,
        func: *mut u8,
        param_count: u32,
        result_count: u32,
    ) -> Result<*mut VMContRef, TrapReason> {
        let caller_vmctx = instance.vmctx();

        let capacity = cmp::max(param_count, result_count);

        let wasmfx_config = unsafe { &*(store.wasmfx_config()) };
        // TODO(frank-emrich) Currently, the general `stack_limit` configuration
        // option of wasmtime is unrelated to the stack size of our fiber stack.
        let stack_size = wasmfx_config.stack_size;
        let red_zone_size = wasmfx_config.red_zone_size;

        let (contref, mut stack) = instance.wasmfx_allocate_continuation().map_err(|_error| {
            TrapReason::User(anyhow::anyhow!("Fiber stack allocation failed!"))
        })?;

        let tsp = stack.top().unwrap();
        let stack_limit = unsafe { tsp.sub(stack_size - red_zone_size) } as usize;
        let limits = StackLimits::with_stack_limit(stack_limit);

        {
            let contref = unsafe { contref.as_mut().unwrap() };
            let csi = &mut contref.common_stack_information;
            csi.limits = limits;
            csi.state = State::Fresh;
            contref.parent_chain = StackChain::Absent;
            contref.args.ensure_capacity(capacity);
            // The continuation is fresh, which is a special case of being suspended.
            // Thus we need to set the correct end of the continuation chain: itself.
            contref.last_ancestor = contref;

            // In order to give the pool a uniform interface for the optimized
            // and baseline implementation, it returns the `FiberStack` as a
            // standalone value, without being attached to the `VMContRef`.
            // We attach them here, the previous `FiberStack` attached to the
            // `VMContRef` while in the pool should be an empty dummy
            // `FiberStack`.
            std::mem::swap(&mut contref.stack, &mut stack);
            debug_assert!(stack.is_unallocated());
            debug_assert!(!contref.stack.is_unallocated());

            contref.stack.initialize(
                func.cast::<VMFuncRef>(),
                caller_vmctx,
                contref.args.data as *mut ValRaw,
                contref.args.capacity as usize,
            );
        };

        // TODO(dhil): we need memory clean up of
        // continuation reference objects.
        debug_println!("Created contref @ {:p}", contref);
        Ok(contref)
    }

    // Tests
    #[test]
    fn offset_and_size_constants() {
        use core::mem::offset_of;
        use wasmtime_continuations::offsets::*;

        assert_eq!(
            offset_of!(VMContRef, common_stack_information),
            vm_cont_ref::COMMON_STACK_INFORMATION
        );
        assert_eq!(
            offset_of!(VMContRef, parent_chain),
            vm_cont_ref::PARENT_CHAIN
        );
        assert_eq!(
            offset_of!(VMContRef, last_ancestor),
            vm_cont_ref::LAST_ANCESTOR
        );
        assert_eq!(offset_of!(VMContRef, stack), vm_cont_ref::STACK);
        assert_eq!(offset_of!(VMContRef, args), vm_cont_ref::ARGS);
        assert_eq!(offset_of!(VMContRef, values), vm_cont_ref::VALUES);

        assert_eq!(offset_of!(VMContRef, revision), vm_cont_ref::REVISION);

        assert_eq!(core::mem::size_of::<FiberStack>(), FIBER_STACK_SIZE);
        assert_eq!(core::mem::size_of::<StackChain>(), STACK_CHAIN_SIZE);

        // `CommonStackInformation` and `StackLimits` offsets don't need tests because
        // they are defined diretly with `offset_of!`
    }
}

#[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))]
pub mod baseline {
    use super::stack_chain::{StackChain, StackLimits};
    use crate::runtime::vm::{Instance, TrapReason, VMFuncRef, VMOpaqueContext, VMStore, ValRaw};
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
        pub fiber: Option<Box<ContinuationFiber>>,
        pub suspend: *mut Yield,
        pub limits: StackLimits,
        pub parent_chain: StackChain,
        pub parent: *mut VMContRef,
        pub args: Vec<ValRaw>,
        pub values: Vec<ValRaw>,
        pub _marker: core::marker::PhantomPinned,
    }

    impl VMContRef {
        pub fn fiber_stack(&self) -> &FiberStack {
            self.fiber.as_ref().unwrap().stack()
        }

        pub fn detach_stack(&mut self) -> FiberStack {
            self.fiber
                .take()
                .expect("Only call detach_stack if a stack is actually present")
                .into_stack()
        }

        /// This is effectively a `Default` implementation, without calling it
        /// so. Used to create `VMContRef`s when initializing pooling allocator.
        pub fn empty() -> Self {
            let limits = StackLimits::with_stack_limit(Default::default());
            let parent_chain = StackChain::Absent;
            let parent = core::ptr::null_mut();
            let suspend = core::ptr::null_mut();
            let args = Vec::new();
            let values = Vec::new();
            let fiber = None;
            let revision = 0;

            Self {
                revision,
                limits,
                parent_chain,
                parent,
                suspend,
                fiber,
                args,
                values,
                _marker: core::marker::PhantomPinned,
            }
        }
    }

    // These are required so the WasmFX pooling allocator can store a Vec of
    // `VMContRef`s.
    unsafe impl Send for VMContRef {}
    unsafe impl Sync for VMContRef {}

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
        _store: &mut dyn VMStore,
        instance: &mut Instance,
        func: *mut u8,
        param_count: usize,
        result_count: usize,
    ) -> Result<*mut VMContRef, TrapReason> {
        let capacity = cmp::max(param_count, result_count);
        let mut values: Vec<ValRaw> = Vec::with_capacity(capacity);

        let (contref, fiber) = {
            let (contref, stack) = instance
                .wasmfx_allocate_continuation()
                .map_err(|error| TrapReason::User(error.into()))?;

            let fiber = match unsafe { func.cast::<VMFuncRef>().as_ref() } {
                None => Fiber::new(stack, |_instance: &mut Instance, _suspend: &mut Yield| {
                    panic!("Attempt to invoke null VMFuncRef!");
                }),
                Some(func_ref) => {
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
                            let _result = func_ref.array_call(
                                None,
                                caller_ctx,
                                std::slice::from_raw_parts_mut(vals_ptr, capacity),
                            );
                            // TODO(dhil): We are ignoring the boolean
                            // return value, which indicates success
                            // or failure. We probably shouldn't
                            // ignore it...

                            // As a precaution we null the suspender.
                            (*get_current_continuation()).suspend = core::ptr::null_mut();
                            return ();
                        },
                    )
                }
            };
            let fiber = fiber.map_err(|error| TrapReason::User(error.into()))?;
            (contref, fiber)
        };

        {
            let contref = unsafe { contref.as_mut().unwrap() };

            debug_assert!(contref.fiber.is_none());
            contref.fiber = Some(Box::new(fiber));

            // We gave the data pointer of `values` to the trampoline and *must*
            // therefore use it.
            contref.values = values;

            contref.args.clear();
            if contref.args.capacity() < param_count {
                contref.args.reserve(param_count - contref.args.capacity());
            }
            contref.limits = StackLimits::with_stack_limit(0);
            contref.parent = core::ptr::null_mut();
            contref.parent_chain = StackChain::Absent;
            contref.suspend = core::ptr::null_mut();

            // Note that we keep the revision counter unchanged.

            // TODO(dhil): we need memory clean up of
            // continuation reference objects.
            debug_assert!(!contref
                .fiber
                .as_ref()
                .unwrap()
                .stack()
                .top()
                .unwrap()
                .is_null());
        };

        Ok(contref)
    }

    /// Continues a given continuation.
    #[inline(always)]
    pub fn resume(
        store: &mut dyn VMStore,
        instance: &mut Instance,
        contref: &mut VMContRef,
    ) -> Result<u32, TrapReason> {
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
        unsafe { *(*store.vmruntime_limits()).stack_limit.get_mut() = 0 };

        // Resume the current continuation.
        contref
            .fiber
            .as_ref()
            .unwrap()
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

        instance.wasmfx_deallocate_continuation(contref);
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
                contref.args.push(ValRaw::v128(0)); // zero initialise
            }
            offset = (contref.args.len() - nargs) as isize;
        }
        unsafe { contref.args.as_mut_ptr().offset(offset).cast::<u128>() }
    }

    /// Returns the pointer to the (return) values buffer of a given
    /// continuation reference.
    #[inline(always)]
    pub fn get_values_ptr(_instance: &mut Instance, contref: &mut VMContRef) -> *mut u128 {
        contref.values.as_mut_ptr().cast::<u128>()
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
    use wasmtime_continuations::CommonStackInformation;
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
        MainStack(*mut CommonStackInformation) =
            wasmtime_continuations::STACK_CHAIN_MAIN_STACK_DISCRIMINANT,
        /// Represents a continuation's stack.
        Continuation(*mut VMContRef) =
            wasmtime_continuations::STACK_CHAIN_CONTINUATION_DISCRIMINANT,
    }

    impl StackChain {
        /// Indicates if `self` is a `MainStack` variant.
        pub fn is_main_stack(&self) -> bool {
            matches!(self, StackChain::MainStack(_))
        }

        /// Returns an iterator over the continuations in this chain.
        /// We don't implement `IntoIterator` because our iterator is unsafe, so at
        /// least this gives us some way of indicating this, even though the actual
        /// unsafety lies in the `next` function.
        ///
        /// # Safety
        ///
        /// This function is not unsafe per see, but it returns an object
        /// whose usage is unsafe.
        pub unsafe fn into_continuation_iter(self) -> ContinuationIterator {
            ContinuationIterator(self)
        }

        /// Returns an iterator over the stack limits in this chain.
        /// We don't implement `IntoIterator` because our iterator is unsafe, so at
        /// least this gives us some way of indicating this, even though the actual
        /// unsafety lies in the `next` function.
        ///
        /// # Safety
        ///
        /// This function is not unsafe per see, but it returns an object
        /// whose usage is unsafe.
        pub unsafe fn into_stack_limits_iter(self) -> StackLimitsIterator {
            StackLimitsIterator(self)
        }
    }

    /// Iterator for Continuations in a stack chain.
    #[cfg_attr(feature = "wasmfx_baseline", allow(dead_code))]
    pub struct ContinuationIterator(StackChain);

    /// Iterator for StackLimits in a stack chain.
    #[cfg_attr(feature = "wasmfx_baseline", allow(dead_code))]
    pub struct StackLimitsIterator(StackChain);

    impl Iterator for ContinuationIterator {
        type Item = *mut VMContRef;

        #[cfg(any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"))]
        fn next(&mut self) -> Option<Self::Item> {
            match self.0 {
                StackChain::Absent | StackChain::MainStack(_) => None,
                StackChain::Continuation(ptr) => {
                    let continuation = unsafe { ptr.as_mut().unwrap() };
                    self.0 = continuation.parent_chain.clone();
                    Some(ptr)
                }
            }
        }

        #[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))]
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
        }
    }

    impl Iterator for StackLimitsIterator {
        type Item = *mut StackLimits;

        #[cfg(any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"))]
        fn next(&mut self) -> Option<Self::Item> {
            match self.0 {
                StackChain::Absent => None,
                StackChain::MainStack(csi) => {
                    let stack_limits = unsafe { &mut (*csi).limits } as *mut StackLimits;
                    self.0 = StackChain::Absent;
                    Some(stack_limits)
                }
                StackChain::Continuation(ptr) => {
                    let continuation = unsafe { ptr.as_mut().unwrap() };
                    let stack_limits =
                        (&mut continuation.common_stack_information.limits) as *mut StackLimits;
                    self.0 = continuation.parent_chain.clone();
                    Some(stack_limits)
                }
            }
        }

        #[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))]
        fn next(&mut self) -> Option<Self::Item> {
            unimplemented!()
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
#[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))]
pub mod optimized {
    use crate::runtime::vm::{Instance, TrapReason, VMStore};

    pub type VMContRef = super::baseline::VMContRef;

    #[inline(always)]
    pub fn drop_cont_ref(_instance: &mut Instance, _contref: *mut VMContRef) {
        panic!("attempt to execute continuation::optimized::drop_cont_ref with `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    pub fn cont_new(
        _store: &mut dyn VMStore,
        _instance: &mut Instance,
        _func: *mut u8,
        _param_count: u32,
        _result_count: u32,
    ) -> Result<*mut VMContRef, TrapReason> {
        panic!("attempt to execute continuation::optimized::cont_new with `typed_continuation_baseline_implementation` toggled!")
    }
}

#[allow(missing_docs)]
#[cfg(any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"))]
pub mod baseline {
    use crate::runtime::vm::{Instance, TrapReason, VMStore};

    #[allow(missing_docs)]
    #[repr(C)]
    pub struct VMContRef();

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn cont_new(
        _store: &mut dyn VMStore,
        _instance: &mut Instance,
        _func: *mut u8,
        _param_count: usize,
        _result_count: usize,
    ) -> Result<*mut VMContRef, TrapReason> {
        panic!("attempt to execute continuation::baseline::cont_new without `typed_continuation_baseline_implementation` toggled!")
    }

    #[inline(always)]
    #[allow(missing_docs)]
    pub fn resume(
        _store: &mut dyn VMStore,
        _instance: &mut Instance,
        _contref: &mut VMContRef,
    ) -> Result<u32, TrapReason> {
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
