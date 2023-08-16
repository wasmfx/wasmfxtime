//! Continuations TODO

use crate::instance::TopOfStackPointer;
use crate::vmcontext::{VMArrayCallFunction, VMFuncRef, VMOpaqueContext, ValRaw};
use crate::{Instance, TrapReason};
use std::cmp;
use std::mem;
use std::ptr;
use wasmtime_fibre::{Fiber, FiberStack, Suspend};

type ContinuationFiber = Fiber<'static, (), u32, ()>;
type Yield = Suspend<(), u32, ()>;

#[allow(dead_code)]
const ENABLE_DEBUG_PRINTING: bool = false;

macro_rules! debug_println {
    ($( $args:expr ),+ ) => {
        #[cfg(debug_assertions)]
        if ENABLE_DEBUG_PRINTING {
            println!($($args),*);
        }
    }
}

struct Payloads {
    length: usize,
    capacity: usize,
    /// This is null if and only if capacity (and thus also `length`) are 0.
    data: *mut u128,
}

impl Payloads {
    fn new(capacity: usize) -> Payloads {
        let data = if capacity == 0 {
            ptr::null_mut()
        } else {
            let mut args = Vec::with_capacity(capacity);
            let args_ptr = args.as_mut_ptr();
            args.leak();
            args_ptr
        };
        return Payloads {
            length: 0,
            capacity,
            data,
        };
    }

    fn occupy_next(&mut self, count: usize) -> *mut u128 {
        let original_length = self.length;
        assert!(self.length + count <= self.capacity);
        self.length += count;
        return unsafe { self.data.offset(original_length as isize) };
    }
}

/// Encodes the life cycle of a `ContinuationObject`.
#[derive(PartialEq)]
enum State {
    /// The `ContinuationObject` has been created, but `resume` has never been
    /// called on it. During this stage, we may add arguments using `cont.bind`.
    Allocated,
    /// `resume` has been invoked at least once on the `ContinuationObject`,
    /// meaning that the function passed to `cont.new` has started executing.
    /// Note that this state does not indicate whether the execution of this
    /// function is currently suspended or not.
    Invoked,
    /// The function originally passed to `cont.new` has returned normally.
    /// Note that there is no guarantee that a ContinuationObject will ever
    /// reach this status, as it may stay suspended until being dropped.
    Returned,
}

/// TODO
#[repr(C)]
pub struct ContinuationObject {
    fiber: *mut ContinuationFiber,

    /// Used to store
    /// 1. The arguments to the function passed to cont.new
    /// 2. The return values of that function
    /// Note that this is *not* used for tag payloads.
    args: Payloads,

    // Once a continuation is suspended, this buffer is used to hold payloads
    // provided by cont.bind and resume and received at the suspend site.
    // In particular, this may only be Some when `state` is `Invoked`.
    tag_return_values: Option<Box<Payloads>>,

    state: State,
}

/// M:1 Many-to-one mapping. A single ContinuationObject may be
/// referenced by multiple ContinuationReference, though, only one
/// ContinuationReference may hold a non-null reference to the object
/// at a given time.
#[repr(C)]
pub struct ContinuationReference(Option<*mut ContinuationObject>);

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
#[inline(always)]
pub fn cont_obj_get_results(obj: *mut ContinuationObject) -> *mut u128 {
    assert!(unsafe { (*obj).state == State::Returned });
    assert!(unsafe { !(*obj).args.data.is_null() });
    unsafe { (*obj).args.data }
}

/// TODO
#[inline(always)]
pub fn cont_obj_occupy_next_args_slots(
    obj: *mut ContinuationObject,
    arg_count: usize,
) -> *mut u128 {
    assert!(unsafe { (*obj).state == State::Allocated });
    let args = &mut unsafe { obj.as_mut() }.unwrap().args;
    return args.occupy_next(arg_count);
}

/// TODO
#[inline(always)]
pub fn cont_obj_occupy_next_tag_returns_slots(
    obj: *mut ContinuationObject,
    arg_count: usize,
    remaining_arg_count: usize,
) -> *mut u128 {
    let obj = unsafe { obj.as_mut().unwrap() };
    assert!(obj.state == State::Invoked);
    let payloads = obj
        .tag_return_values
        .get_or_insert_with(|| Box::new(Payloads::new(remaining_arg_count)));
    return payloads.occupy_next(arg_count);
}

/// TODO
pub fn cont_obj_get_tag_return_values_buffer(
    obj: *mut ContinuationObject,
    expected_value_count: usize,
) -> *mut u128 {
    let obj = unsafe { obj.as_mut().unwrap() };
    assert!(obj.state == State::Invoked);

    let payloads = &mut obj.tag_return_values.as_ref().unwrap();
    assert_eq!(payloads.length, expected_value_count);
    assert_eq!(payloads.length, payloads.capacity);
    assert!(!payloads.data.is_null());
    return payloads.data;
}

/// TODO
pub fn cont_obj_deallocate_tag_return_values_buffer(obj: *mut ContinuationObject) {
    let obj = unsafe { obj.as_mut().unwrap() };
    assert!(obj.state == State::Invoked);
    let payloads: Box<Payloads> = obj.tag_return_values.take().unwrap();
    let _: Vec<u128> =
        unsafe { Vec::from_raw_parts((*payloads).data, (*payloads).length, (*payloads).capacity) };
    obj.tag_return_values = None;
}

/// TODO
#[inline(always)]
pub fn cont_obj_has_state_invoked(obj: *mut ContinuationObject) -> bool {
    // We use this function to determine whether a contination object is in initialisation mode or
    // not.
    // FIXME(frank-emrich) Rename this function to make it clearer that we shouldn't call
    // it in `Returned` state.
    assert!(unsafe { (*obj).state != State::Returned });

    return unsafe { (*obj).state == State::Invoked };
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
    let contobj: Box<ContinuationObject> = unsafe { Box::from_raw(contobj) };
    let _: Box<ContinuationFiber> = unsafe { Box::from_raw(contobj.fiber) };
    unsafe {
        let _: Vec<u128> = Vec::from_raw_parts(
            contobj.args.data,
            contobj.args.length,
            contobj.args.capacity,
        );
    };
    match contobj.tag_return_values {
        None => (),
        Some(payloads) => unsafe {
            let _: Vec<u128> =
                Vec::from_raw_parts(payloads.data, payloads.length, payloads.capacity);
        },
    }
}

/// TODO
pub fn allocate_payload_buffer(instance: &mut Instance, element_count: usize) -> *mut u128 {
    // In the current design, we allocate a `Vec<u128>` and store a pointer to
    // it in the `VMContext` payloads pointer slot. We then return the pointer
    // to the `Vec`'s data, not to the `Vec` itself.
    // This is mostly for debugging purposes, since the `Vec` stores its size.
    // Alternatively, we may allocate the buffer ourselves here and store the
    // pointer directly in the `VMContext`. This would avoid one level of
    // pointer indirection.

    let payload_ptr =
        unsafe { instance.get_typed_continuations_payloads_ptr_mut() as *mut *mut Vec<u128> };

    // FIXME(frank-emrich) This doesn't work, yet, because we don't zero-initialize the
    // payload pointer field in the VMContext, meaning that it may initially contain garbage.
    // Ensure that there isn't an active payload buffer. If there was, we didn't clean up propertly
    // assert!(unsafe { (*payload_ptr).is_null() });

    let mut vec = Box::new(Vec::<u128>::with_capacity(element_count));

    let vec_data = (*vec).as_mut_ptr();
    unsafe {
        *payload_ptr = Box::into_raw(vec);
    }
    return vec_data;
}

/// TODO
pub fn deallocate_payload_buffer(instance: &mut Instance, element_count: usize) {
    let payload_ptr =
        unsafe { instance.get_typed_continuations_payloads_ptr_mut() as *mut *mut Vec<u128> };

    let vec = unsafe { Box::from_raw(*payload_ptr) };

    // If these don't match something went wrong.
    assert_eq!(vec.capacity(), element_count);

    unsafe { *payload_ptr = ptr::null_mut() };

    // payload buffer destroyed when `vec` goes out of scope
}

/// TODO
pub fn get_payload_buffer(instance: &mut Instance, element_count: usize) -> *mut u128 {
    let payload_ptr =
        unsafe { instance.get_typed_continuations_payloads_ptr_mut() as *mut *mut Vec<u128> };

    let vec = unsafe { (*payload_ptr).as_mut().unwrap() };

    // If these don't match something went wrong.
    assert_eq!(vec.capacity(), element_count);

    let vec_data = vec.as_mut_ptr();
    return vec_data;
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
        args: payload,
        tag_return_values: None,
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
    let fiber_stack = unsafe { &fiber.as_ref().unwrap().stack() };
    let tsp = TopOfStackPointer::as_raw(instance.tsp());
    unsafe { fiber_stack.write_parent(tsp) };
    instance.set_tsp(TopOfStackPointer::from_raw(fiber_stack.top().unwrap()));
    debug_println!(
        "Resuming contobj @ {:p}, tsp is {:p}, setting it to {:p}",
        contobj,
        tsp,
        fiber_stack.top().unwrap()
    );
    unsafe {
        (*(*(*instance.store()).vmruntime_limits())
            .stack_limit
            .get_mut()) = 0
    };
    unsafe { (*contobj).state = State::Invoked };
    // This is to make sure that after we resume from a suspend, we can load the continuation object
    // to access the tag return values.
    unsafe {
        let cont_store_ptr =
            instance.get_typed_continuations_store_mut() as *mut *mut ContinuationObject;
        cont_store_ptr.write(contobj)
    };
    match unsafe { fiber.as_mut().unwrap().resume(()) } {
        Ok(()) => {
            // The result of the continuation was written to the first
            // entry of the payload store by virtue of using the array
            // calling trampoline to execute it.

            // Restore tsp pointer in instance
            let _tsp = TopOfStackPointer::as_raw(instance.tsp());
            let parent = unsafe { (*(*contobj).fiber).stack().parent() };
            instance.set_tsp(TopOfStackPointer::from_raw(parent));

            debug_println!(
                "Continuation @ {:p} returned normally, setting tsp from {:p} to {:p}",
                contobj,
                _tsp,
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
            unsafe {
                let cont_store_ptr =
                    instance.get_typed_continuations_store_mut() as *mut *mut ContinuationObject;
                cont_store_ptr.write(contobj)
            };
            Ok(tag | signal_mask)
        }
    }
}

/// TODO
#[inline(always)]
pub fn suspend(instance: &mut Instance, tag_index: u32) {
    let stack_ptr = TopOfStackPointer::as_raw(instance.tsp());
    let parent = unsafe { stack_ptr.cast::<*mut u8>().offset(-2).read() };
    debug_println!(
        "Suspending, setting tsp from {:p} to {:p}",
        stack_ptr,
        parent
    );
    instance.set_tsp(TopOfStackPointer::from_raw(parent));
    let suspend = wasmtime_fibre::unix::Suspend::from_top_ptr(stack_ptr);
    suspend.switch::<(), u32, ()>(wasmtime_fibre::RunResult::Yield(tag_index))
}
