//! Continuations TODO

use crate::vmcontext::{VMCallerCheckedFuncRef, VMContext, VMOpaqueContext, VMFunctionBody};
use crate::instance::TopOfStackPointer;
use crate::{TrapReason, prepare_host_to_wasm_trampoline};
use wasmtime_fiber::{Fiber, FiberStack, Suspend};

/// TODO
#[inline(always)]
pub fn cont_new(vmctx: *mut VMContext, func: *mut u8) -> *mut u8 {
    let func = func as *mut VMCallerCheckedFuncRef;
    let fiber = Box::new(
        Fiber::new(
            FiberStack::new(4096).unwrap(),
            move |first_val: (), _suspend: &Suspend<_, u32, _>| {
                let trampoline = unsafe {
                    std::mem::transmute::<
                            *const VMFunctionBody,
                            unsafe extern "C" fn(*mut VMOpaqueContext, *mut VMContext, ()),
                        >((*func).func_ptr.as_ptr())
                };
                let trampoline = unsafe { prepare_host_to_wasm_trampoline(vmctx, trampoline) };
                unsafe { trampoline((*func).vmctx , vmctx, first_val) }
            },
        )
        .unwrap(),
    );
    let ptr: *mut Fiber<'static, (), u32, ()> = Box::into_raw(fiber);
    ptr as *mut _
}

/// TODO
#[inline(always)]
pub fn resume(vmctx: *mut VMContext, cont: *mut u8) -> Result<u32, TrapReason> {
    let inst = unsafe { vmctx.as_mut().unwrap().instance_mut() };
    let cont = cont as *mut Fiber<'static, (), u32, ()>;
    let cont_stack = unsafe { &cont.as_ref().unwrap().stack() };
    let tsp = TopOfStackPointer::as_raw(inst.tsp());
    unsafe { cont_stack.write_parent(tsp) };
    inst.set_tsp(TopOfStackPointer::from_raw(cont_stack.top().unwrap()));
    unsafe {
        (*(*(*(*vmctx).instance().store()).vmruntime_limits())
         .stack_limit
         .get_mut()) = 0
    };
    match unsafe { cont.as_mut().unwrap().resume(()) } {
        Ok(_) => Ok(9999),
        Err(y) => Ok(y),
    }
}

/// TODO
#[inline(always)]
pub fn suspend(vmctx: *mut VMContext, tag_index: u32) {
    let inst = unsafe { vmctx.as_mut().unwrap().instance_mut() };
    let stack_ptr = TopOfStackPointer::as_raw(inst.tsp());
    let parent = unsafe { stack_ptr.cast::<*mut u8>().offset(-2).read() };
    inst.set_tsp(TopOfStackPointer::from_raw(parent));
    let suspend = wasmtime_fiber::unix::Suspend::from_top_ptr(stack_ptr);
    suspend.switch::<(), u32, ()>(wasmtime_fiber::RunResult::Yield(tag_index))
}
