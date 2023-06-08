//! Continuations TODO

use crate::instance::TopOfStackPointer;
use crate::vmcontext::{VMContext, VMFuncRef, VMOpaqueContext, VMWasmCallFunction};
use crate::{Instance, TrapReason};
use std::ptr::NonNull;
use wasmtime_fibre::{Fiber, FiberStack, Suspend};

/// TODO
#[inline(always)]
pub fn cont_new(instance: &mut Instance, func: *mut u8) -> *mut u8 {
    let func = func as *mut VMFuncRef;
    let callee_ctx = unsafe { (*func).vmctx };
    let caller_ctx = instance.vmctx();
    let f = unsafe {
        // TODO(dhil): Not sure whether we should use
        // VMWasmCallFunction or VMNativeCallFunction here.
        std::mem::transmute::<
            NonNull<VMWasmCallFunction>,
            unsafe extern "C" fn(*mut VMOpaqueContext, *mut VMContext, ()) -> u32,
        >((*func).wasm_call.unwrap())
    };
    let fiber = Box::new(
        Fiber::new(
            FiberStack::new(4096).unwrap(),
            move |_first_val: (), _suspend: &Suspend<(), u32, u32>| {
                // TODO(frank-emrich): Need to load arguments (if present) from
                // payload storage and pass to f.
                // Consider getting the array_call version from func
                // to achieve this instead.
                unsafe { f(callee_ctx, caller_ctx, ()) }
            },
        )
        .unwrap(),
    );
    let ptr: *mut Fiber<'static, (), u32, u32> = Box::into_raw(fiber);
    ptr as *mut u8
}

/// TODO
#[inline(always)]
pub fn resume(instance: &mut Instance, cont: *mut u8) -> Result<u32, TrapReason> {
    let cont = cont as *mut Fiber<'static, (), u32, u32>;
    let cont_stack = unsafe { &cont.as_ref().unwrap().stack() };
    let tsp = TopOfStackPointer::as_raw(instance.tsp());
    unsafe { cont_stack.write_parent(tsp) };
    instance.set_tsp(TopOfStackPointer::from_raw(cont_stack.top().unwrap()));
    unsafe {
        (*(*(*instance.store()).vmruntime_limits())
            .stack_limit
            .get_mut()) = 0
    };
    match unsafe { cont.as_mut().unwrap().resume(()) } {
        Ok(result) => {
            let drop_box: Box<Fiber<_, _, _>> = unsafe { Box::from_raw(cont) };
            drop(drop_box); // I think this would be covered by the close brace below anyway
                            // Store the result.
            let payloads_addr = unsafe { instance.get_typed_continuations_payloads_mut() };
            unsafe {
                std::ptr::write(payloads_addr, result);
            }

            Ok(0) // zero value = return normally.
                  //Ok(9999)
        }
        Err(tag) => {
            // We set the high bit to signal a return via suspend. We
            // encode the tag into the remainder of the integer.
            let signal_mask = 0xf000_0000;
            debug_assert_eq!(tag & signal_mask, 0);
            unsafe {
                let cont_store_ptr = instance.get_typed_continuations_store_mut()
                    as *mut *mut Fiber<'static, (), u32, u32>;
                cont_store_ptr.write(cont)
            };
            Ok(tag | signal_mask)
        } // 0 = suspend //Ok(y),
    }
}

/// TODO
#[inline(always)]
pub fn suspend(instance: &mut Instance, tag_index: u32) {
    let stack_ptr = TopOfStackPointer::as_raw(instance.tsp());
    let parent = unsafe { stack_ptr.cast::<*mut u8>().offset(-2).read() };
    instance.set_tsp(TopOfStackPointer::from_raw(parent));
    let suspend = wasmtime_fibre::unix::Suspend::from_top_ptr(stack_ptr);
    suspend.switch::<(), u32, ()>(wasmtime_fibre::RunResult::Yield(tag_index))
}
