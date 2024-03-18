//! Continuations TODO

use crate::fibre::{Fiber, FiberStack, Suspend};
use crate::vmcontext::{VMFuncRef, VMOpaqueContext, ValRaw};
use crate::{Instance, TrapReason};
use std::cell::UnsafeCell;
use std::cmp;
use std::mem;
use wasmtime_continuations::{debug_println, ENABLE_DEBUG_PRINTING};
pub use wasmtime_continuations::{
    Payloads, StackLimits, State, SwitchDirection, DEFAULT_FIBER_SIZE,
};

type Yield = Suspend;

/// Fibers used for continuations
pub type ContinuationFiber = Fiber;

/// This type represents a linked lists of stacks, additionally associating a
/// `StackLimits` object with each element of the list. Here, a "stack" is
/// either a continuation or the main stack. Note that the linked list character
/// arises from the fact that `StackChain::Continuation` variants have a pointer
/// to have `ContinuationObject`, which in turn has a `parent_chain` value of
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
/// fields of the corresponding `ContinuationObject`. For the main stack, the
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
    /// has not been set, yet. If stored in a ContinuationObject's parent_chain
    /// field, means that there is currently no parent.
    Absent = wasmtime_continuations::STACK_CHAIN_ABSENT_DISCRIMINANT,
    /// Represents the main stack.
    MainStack(*mut StackLimits) = wasmtime_continuations::STACK_CHAIN_MAIN_STACK_DISCRIMINANT,
    /// Represents a continuation's stack.
    Continuation(*mut ContinuationObject) =
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
/// corresponding `ContinuationObject`, or None for the main stack.
pub struct ContinuationChainIterator(StackChain);

impl Iterator for ContinuationChainIterator {
    type Item = (Option<*mut ContinuationObject>, *mut StackLimits);

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

/// TODO
#[repr(C)]
pub struct ContinuationObject {
    /// The limits of this continuation's stack.
    pub limits: StackLimits,

    /// The parent of this continuation, which may be another continuation, the
    /// main stack, or absent (in case of a suspended continuation).
    pub parent_chain: StackChain,

    /// The underlying `Fiber`.
    pub fiber: *mut ContinuationFiber,

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
}

/// M:1 Many-to-one mapping. A single ContinuationObject may be
/// referenced by multiple ContinuationReference, though, only one
/// ContinuationReference may hold a non-null reference to the object
/// at a given time.
#[repr(C)]
pub struct ContinuationReference(pub Option<*mut ContinuationObject>);

/// TODO
#[inline(always)]
pub fn cont_ref_get_cont_obj(
    contref: *mut ContinuationReference,
) -> Result<*mut ContinuationObject, TrapReason> {
    //FIXME rename to indicate that this invalidates the cont ref

    // If this is enabled, we should never call this function.
    assert!(!cfg!(
        feature = "unsafe_disable_continuation_linearity_check"
    ));

    let contopt = unsafe {
        contref
            .as_mut()
            .ok_or_else(|| {
                TrapReason::user_without_backtrace(anyhow::anyhow!(
                    "Attempt to dereference null ContinuationReference!"
                ))
            })?
            .0
    };
    match contopt {
        None => Err(TrapReason::user_without_backtrace(anyhow::Error::msg(
            "Continuation is already taken",
        ))), // TODO(dhil): presumably we can set things up such that
        // we always read from a non-null reference.
        Some(contobj) => {
            unsafe {
                *contref = ContinuationReference(None);
            }
            Ok(contobj.cast::<ContinuationObject>())
        }
    }
}

/// TODO
pub fn cont_obj_forward_tag_return_values_buffer(
    parent: *mut ContinuationObject,
    child: *mut ContinuationObject,
) -> Result<(), TrapReason> {
    let parent = unsafe {
        parent.as_mut().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Attempt to dereference null (parent) ContinuationObject"
            ))
        })?
    };
    let child = unsafe {
        child.as_mut().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Attempt to dereference null (child) ContinuationObject"
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
pub fn new_cont_ref(contobj: *mut ContinuationObject) -> *mut ContinuationReference {
    // If this is enabled, we should never call this function.
    assert!(!cfg!(
        feature = "unsafe_disable_continuation_linearity_check"
    ));

    let contref = Box::new(ContinuationReference(Some(contobj)));
    Box::into_raw(contref)
}

/// TODO
#[inline(always)]
pub fn drop_cont_obj(contobj: *mut ContinuationObject) {
    // Note that continuation objects do not own their parents, hence we ignore
    // parent fields here.

    let contobj: Box<ContinuationObject> = unsafe { Box::from_raw(contobj) };
    let _: Box<ContinuationFiber> = unsafe { Box::from_raw(contobj.fiber) };
    unsafe {
        let _: Vec<u128> = Vec::from_raw_parts(
            contobj.args.data,
            contobj.args.length as usize,
            contobj.args.capacity as usize,
        );
    };
    let payloads = &contobj.tag_return_values;
    let _: Vec<u128> = unsafe {
        Vec::from_raw_parts(
            payloads.data,
            payloads.length as usize,
            payloads.capacity as usize,
        )
    };
}

/// TODO
#[inline(always)]
pub fn cont_new(
    instance: &mut Instance,
    func: *mut u8,
    param_count: u32,
    result_count: u32,
) -> Result<*mut ContinuationObject, TrapReason> {
    let func_ref = unsafe {
        func.cast::<VMFuncRef>().as_ref().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Attempt to dereference null VMFuncRef"
            ))
        })?
    };
    let callee_ctx = func_ref.vmctx;
    let caller_ctx = VMOpaqueContext::from_vmcontext(instance.vmctx());

    let capacity = cmp::max(param_count, result_count);
    let payload = Payloads::new(capacity);

    let fiber = {
        let wasmfx_config = unsafe { &*(*instance.store()).wasmfx_config() };
        let stack = FiberStack::malloc(wasmfx_config.stack_size)
            .map_err(|error| TrapReason::user_without_backtrace(error.into()))?;
        let args_ptr = payload.data;
        let fiber = Fiber::new(stack, move |_first_val: (), _suspend: &Yield| unsafe {
            (func_ref.array_call)(
                callee_ctx,
                caller_ctx,
                args_ptr.cast::<ValRaw>(),
                capacity as usize,
            )
        })
        .map_err(|error| TrapReason::user_without_backtrace(error.into()))?;
        Box::new(fiber)
    };

    let tsp = fiber.stack().top().unwrap();
    let contobj = Box::new(ContinuationObject {
        limits: StackLimits::with_stack_limit(unsafe { tsp.sub(DEFAULT_FIBER_SIZE) } as usize),
        fiber: Box::into_raw(fiber),
        parent_chain: StackChain::Absent,
        args: payload,
        tag_return_values: Payloads::new(0),
        state: State::Allocated,
    });

    // TODO(dhil): we need memory clean up of
    // continuation reference objects.
    let pointer = Box::into_raw(contobj);
    debug_println!("Created contobj @ {:p}", pointer);
    Ok(pointer)
}

/// TODO
#[inline(always)]
pub fn resume(
    instance: &mut Instance,
    contobj: *mut ContinuationObject,
    parent_stack_limits: *mut StackLimits,
) -> Result<SwitchDirection, TrapReason> {
    let cont = unsafe {
        contobj.as_ref().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Attempt to dereference null ContinuationObject!"
            ))
        })?
    };
    assert!(cont.state == State::Allocated || cont.state == State::Invoked);
    let fiber = unsafe {
        cont.fiber.as_mut().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Attempt to dereference null Fiber!"
            ))
        })?
    };

    if ENABLE_DEBUG_PRINTING {
        let chain = instance.typed_continuations_stack_chain();
        // SAFETY: We maintain as an invariant that the stack chain field in the
        // VMContext is non-null and contains a chain of zero or more
        // StackChain::Continuation values followed by StackChain::Main.
        match unsafe { (**chain).0.get_mut() } {
            StackChain::Continuation(running_contobj) => {
                debug_assert_eq!(contobj, *running_contobj);
                debug_println!(
                    "Resuming contobj @ {:p}, previously running contobj is {:p}",
                    contobj,
                    running_contobj
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

        *runtime_limits.stack_limit.get() = (*contobj).limits.stack_limit;
        *runtime_limits.last_wasm_entry_sp.get() = (*contobj).limits.last_wasm_entry_sp;
    }

    unsafe {
        (*(*(*instance.store()).vmruntime_limits())
            .stack_limit
            .get_mut()) = 0
    };

    Ok(fiber.resume())
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
        StackChain::MainStack { .. } => Err(TrapReason::user_without_backtrace(anyhow::anyhow!(
            "Calling suspend outside of a continuation"
        ))),
        StackChain::Continuation(running) => {
            // SAFETY: See above.
            Ok(unsafe { &**running })
        }
    }?;

    let fiber = unsafe {
        // SAFETY: See above.
        (*running).fiber.as_ref().ok_or_else(|| {
            TrapReason::user_without_backtrace(anyhow::anyhow!(
                "Attempt to dereference null fiber!"
            ))
        })?
    };

    let stack_ptr = fiber.stack().top().ok_or_else(|| {
        TrapReason::user_without_backtrace(anyhow::anyhow!("Failed to retrieve stack top pointer!"))
    })?;
    debug_println!(
        "Suspending while running {:p}, parent is {:?}",
        running,
        running.parent_chain
    );

    let suspend = crate::fibre::unix::Suspend::from_top_ptr(stack_ptr);
    let payload = SwitchDirection::suspend(tag_index);
    Ok(suspend.switch(payload))
}

#[allow(missing_docs)]
#[cfg(feature = "typed_continuations_baseline_implementation")]
pub mod baseline {
    use crate::{Instance, TrapReason, VMFuncRef, VMOpaqueContext, ValRaw};
    use std::{cell::Cell, cell::RefCell, cmp, mem};
    use wasmtime_fiber::{Fiber, Suspend};

    type ContinuationFiber = Fiber<'static, &'static mut Instance, u32, ()>;
    type Yield = Suspend<&'static mut Instance, u32, ()>;

    /// The baseline VM continuation record.
    ///
    /// It is a linked list of continuation records. Each element in
    /// the list consists of a pointer to an actual
    /// wasmtime_fiber::Fiber, a suspend object, a parent pointer, an
    /// arguments buffer, and a return buffer.
    pub struct VMContRef {
        pub fiber: Box<ContinuationFiber>,
        pub suspend: *const Yield,
        pub parent: *mut VMContRef,
        pub args: Vec<u128>,
        pub values: Vec<u128>,
        pub _marker: std::marker::PhantomPinned,
    }

    // We use thread local state to simulate the VMContext. The use of
    // thread local state is necessary to reliably pass the testsuite,
    // as the test driver is multi-threaded.
    thread_local! {
        // The current continuation, i.e. the currently executing
        // continuation.
        static CC: Cell<*mut VMContRef> = Cell::new(std::ptr::null_mut());
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
        _instance: &mut Instance,
        func: *mut u8,
        param_count: usize,
        result_count: usize,
    ) -> Result<*mut VMContRef, TrapReason> {
        let func_ref = unsafe {
            func.cast::<VMFuncRef>().as_ref().ok_or_else(|| {
                TrapReason::user_without_backtrace(anyhow::anyhow!(
                    "Attempt to dereference null VMFuncRef"
                ))
            })?
        };

        let capacity = cmp::max(param_count, result_count);
        let mut values: Vec<u128> = Vec::with_capacity(capacity);

        let fiber = {
            let callee_ctx = func_ref.vmctx;
            let stack = wasmtime_fiber::FiberStack::new(crate::continuation::DEFAULT_FIBER_SIZE)
                .map_err(|error| TrapReason::user_without_backtrace(error.into()))?;
            let vals_ptr = values.as_mut_ptr();
            let fiber = Fiber::new(
                stack,
                move |instance: &mut Instance, suspend: &Yield| unsafe {
                    let caller_ctx = VMOpaqueContext::from_vmcontext(instance.vmctx());
                    // NOTE(dhil): The cast `suspend as *const Yield`
                    // side-steps the need for mentioning the lifetime
                    // of `Yield`. In this case it is safe, because
                    // Yield lives as long as the object it is
                    // embedded in.
                    (*get_current_continuation()).suspend = suspend as *const Yield;
                    let results = (func_ref.array_call)(
                        callee_ctx,
                        caller_ctx,
                        vals_ptr.cast::<ValRaw>(),
                        capacity,
                    );
                    // As a precaution we null the suspender.
                    (*get_current_continuation()).suspend = std::ptr::null();
                    return results;
                },
            )
            .map_err(|error| TrapReason::user_without_backtrace(error.into()))?;
            Box::new(fiber)
        };

        let contref = Box::new(VMContRef {
            parent: std::ptr::null_mut(),
            suspend: std::ptr::null(),
            fiber,
            args: Vec::with_capacity(param_count),
            values,
            _marker: std::marker::PhantomPinned,
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
        let parent = mem::replace(&mut contref.parent, std::ptr::null_mut());
        set_current_continuation(parent);
        unsafe { contref.suspend.as_ref().unwrap().suspend(tag_index) };
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
    pub fn drop_continuation_reference(_instance: &mut Instance, contref: *mut VMContRef) {
        // Note that continuation objects do not own their parents, so
        // we let the parent object leak.
        let contref: Box<VMContRef> = unsafe { Box::from_raw(contref) };
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

#[allow(missing_docs)]
#[cfg(not(feature = "typed_continuations_baseline_implementation"))]
pub mod baseline {
    use crate::{Instance, TrapReason};

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

#[test]
fn offset_and_size_constants() {
    use memoffset;
    use wasmtime_continuations::offsets::*;

    assert_eq!(
        memoffset::offset_of!(ContinuationObject, limits),
        continuation_object::LIMITS
    );
    assert_eq!(
        memoffset::offset_of!(ContinuationObject, parent_chain),
        continuation_object::PARENT_CHAIN
    );
    assert_eq!(
        memoffset::offset_of!(ContinuationObject, fiber),
        continuation_object::FIBER
    );
    assert_eq!(
        memoffset::offset_of!(ContinuationObject, args),
        continuation_object::ARGS
    );
    assert_eq!(
        memoffset::offset_of!(ContinuationObject, tag_return_values),
        continuation_object::TAG_RETURN_VALUES
    );
    assert_eq!(
        memoffset::offset_of!(ContinuationObject, state),
        continuation_object::STATE
    );

    assert_eq!(std::mem::size_of::<StackChain>(), STACK_CHAIN_SIZE);
}
