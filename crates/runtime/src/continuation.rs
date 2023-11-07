//! Continuations TODO

use crate::vmcontext::{VMArrayCallFunction, VMFuncRef, VMOpaqueContext, ValRaw};
use crate::{Instance, TrapReason};
//use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::cmp;
use std::mem;
use std::ptr;
use wasmtime_continuations::{debug_println, ENABLE_DEBUG_PRINTING};
pub use wasmtime_continuations::{
    ContinuationFiber, ContinuationObject, ContinuationReference, Payloads, State,
};
use wasmtime_fibre::{Fiber, FiberStack, Suspend};

type Yield = Suspend<(), u32, ()>;

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

    let contopt = unsafe { contref.as_mut().unwrap().0 };
    match contopt {
        None => Err(TrapReason::user_with_backtrace(anyhow::Error::msg(
            "Continuation is already taken",
        ))), // TODO(dhil): presumably we can set things up such that
        // we always read from a non-null reference.
        Some(contobj) => {
            unsafe {
                *contref = ContinuationReference(None);
            }
            Ok(contobj as *mut ContinuationObject)
        }
    }
}

/// TODO
pub fn cont_obj_forward_tag_return_values_buffer(
    parent: *mut ContinuationObject,
    child: *mut ContinuationObject,
) {
    let parent = unsafe { parent.as_mut().unwrap() };
    let child = unsafe { child.as_mut().unwrap() };
    assert!(parent.state == State::Invoked);
    assert!(child.state == State::Invoked);

    assert!(child.tag_return_values.capacity == 0);

    mem::swap(&mut child.tag_return_values, &mut parent.tag_return_values);
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
            contobj.args.length,
            contobj.args.capacity,
        );
    };
    let payloads = &contobj.tag_return_values;
    let _: Vec<u128> =
        unsafe { Vec::from_raw_parts(payloads.data, payloads.length, payloads.capacity) };
}

/// TODO
#[inline(always)]
pub fn cont_new(
    instance: &mut Instance,
    func: *mut u8,
    param_count: usize,
    result_count: usize,
) -> *mut ContinuationObject {
    let func = func as *mut VMFuncRef;
    let callee_ctx = unsafe { (*func).vmctx };
    let caller_ctx = VMOpaqueContext::from_vmcontext(instance.vmctx());
    let f = unsafe {
        mem::transmute::<
            VMArrayCallFunction,
            unsafe extern "C" fn(*mut VMOpaqueContext, *mut VMOpaqueContext, *mut ValRaw, usize),
        >((*func).array_call)
    };
    let capacity = cmp::max(param_count, result_count);

    let payload = Payloads::new(capacity);

    let args_ptr = payload.data;
    let fiber = Box::new(
        Fiber::new(
            FiberStack::malloc(4096).unwrap(),
            move |_first_val: (), _suspend: &Yield| unsafe {
                f(callee_ctx, caller_ctx, args_ptr as *mut ValRaw, capacity)
            },
        )
        .unwrap(),
    );

    let contobj = Box::new(ContinuationObject {
        fiber: Box::into_raw(fiber),
        parent: ptr::null_mut(),
        args: payload,
        tag_return_values: Payloads::new(0),
        state: State::Allocated,
    });

    // TODO(dhil): we need memory clean up of
    // continuation reference objects.
    let pointer = Box::into_raw(contobj);
    debug_println!("Created contobj @ {:p}", pointer);
    return pointer;
}

/// TODO
#[inline(always)]
pub fn resume(
    instance: &mut Instance,
    contobj: *mut ContinuationObject,
) -> Result<u32, TrapReason> {
    assert!(unsafe { (*contobj).state == State::Allocated || (*contobj).state == State::Invoked });
    let fiber = unsafe { (*contobj).fiber };

    if ENABLE_DEBUG_PRINTING {
        let _running_contobj = instance.typed_continuations_store();
        debug_println!(
            "Resuming contobj @ {:p}, previously running contobj is {:p}",
            contobj,
            _running_contobj
        );
    }

    // Note that this function updates the typed continuation store field in the
    // VMContext (i.e., the currently running continuation), but does not update
    // any parent pointers. The latter has to happen elsewhere.

    // We mark `contobj` as the currently running one
    instance.set_typed_continuations_store(contobj);

    unsafe {
        (*(*(*instance.store()).vmruntime_limits())
            .stack_limit
            .get_mut()) = 0
    };
    unsafe { (*contobj).state = State::Invoked };

    match unsafe { fiber.as_mut().unwrap().resume(()) } {
        Ok(()) => {
            // The result of the continuation was written to the first
            // entry of the payload store by virtue of using the array
            // calling trampoline to execute it.

            // Restore the currently running contobj entry in the VMContext
            let parent = unsafe { (*contobj).parent };
            instance.set_typed_continuations_store(parent);

            debug_println!(
                "Continuation @ {:p} returned normally, setting running continuation in VMContext to {:p}",
                contobj,
                parent
            );

            unsafe { (*contobj).state = State::Returned };
            Ok(0) // zero value = return normally.
        }
        Err(tag) => {
            debug_println!("Continuation {:p} suspended", contobj);

            // We set the high bit to signal a return via suspend. We
            // encode the tag into the remainder of the integer.
            let signal_mask = 0xf000_0000;
            debug_assert_eq!(tag & signal_mask, 0);

            // Restore the currently running contobj entry in the VMContext
            let parent = unsafe { (*contobj).parent };
            instance.set_typed_continuations_store(parent);

            Ok(tag | signal_mask)
        }
    }
}

/// TODO
#[inline(always)]
pub fn suspend(instance: &mut Instance, tag_index: u32) {
    let running = instance.typed_continuations_store();
    let running = unsafe {
        running
            .as_ref()
            .expect("Calling suspend outside of a continuation") // TODO(dhil): we should emit the trap UnhandledTag here.
    };

    let stack_ptr = unsafe { (*running.fiber).stack().top().unwrap() };
    debug_println!(
        "Suspending while running {:p}, parent is {:p}",
        running,
        running.parent
    );

    let suspend = wasmtime_fibre::unix::Suspend::from_top_ptr(stack_ptr);
    suspend.switch::<(), u32, ()>(wasmtime_fibre::RunResult::Yield(tag_index))
}
