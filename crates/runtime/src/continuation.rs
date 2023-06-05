//! Continuations TODO

use crate::instance::TopOfStackPointer;
use crate::vmcontext::{VMContext, VMFuncRef, VMFunctionBody, VMOpaqueContext};
use crate::{Instance, TrapReason};
use std::mem;
use wasmtime_fibre::{Fiber, FiberStack, Suspend};

// NOTE(dhil): Trampolines have disappeared from the wasmtime codebase
// in favour of another mechanism. Presently, our code depends on
// trampolines, so I have copied the necessary trampoline code in
// here. We should ultimately fix our code such that we can use the
// new mechanism.

// Trampolines for calling into Wasm from the host and calling the host from
// Wasm.

/// Given a Wasm function pointer and a `vmctx`, prepare the `vmctx` for calling
/// into that Wasm function, and return the host-to-Wasm entry trampoline.
///
/// Callers must never call Wasm function pointers directly. Callers must
/// instead call this function and then enter Wasm through the returned
/// host-to-Wasm trampoline.
///
/// # Unsafety
///
/// The `vmctx` argument must be valid.
///
/// The generic type `T` must be a function pointer type and `func` must be a
/// pointer to a Wasm function of that signature.
///
/// After calling this function, you may not mess with the vmctx or any other
/// Wasm state until after you've called the trampoline returned by this
/// function.
#[inline]
pub unsafe fn prepare_host_to_wasm_trampoline<T>(vmctx: *mut VMContext, func: T) -> T {
    assert_eq!(mem::size_of::<T>(), mem::size_of::<usize>());

    // Save the callee in the `vmctx`. The trampoline will read this function
    // pointer and tail call to it.
    Instance::from_vmctx(vmctx, |instance: &mut Instance| {
        instance.set_callee(Some(mem::transmute_copy(&func)))
    });

    // Give callers the trampoline, transmuted into their desired function
    // signature (the trampoline is variadic and works with all signatures).
    mem::transmute_copy(&(host_to_wasm_trampoline as usize))
}

extern "C" {
    fn host_to_wasm_trampoline();
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        use wasmtime_asm_macros::asm_func;

        // Helper macros for getting the first and second arguments according to the
        // system calling convention, as well as some callee-saved scratch registers we
        // can safely use in the trampolines.
        cfg_if::cfg_if! {
            if #[cfg(unix)] {
                macro_rules! wasmfx_arg0 { () => ("rdi") }
                macro_rules! wasmfx_arg1 { () => ("rsi") }
                macro_rules! wasmfx_scratch0 { () => ("r10") }
                macro_rules! wasmfx_scratch1 { () => ("r11") }
            } else {
                compile_error!("platform not supported");
            }
        }

        #[rustfmt::skip]
        asm_func!(
            "host_to_wasm_trampoline",
            concat!(
                "
            .cfi_startproc simple
            .cfi_def_cfa_offset 0

            // Load the pointer to `VMRuntimeLimits` in `scratch0`.
            mov ", wasmfx_scratch0!(), ", 8[", wasmfx_arg1!(), "]

            // Check to see if this is a core `VMContext` (MAGIC == 'core').
            cmp DWORD PTR [", wasmfx_arg0!(), "], 0x65726f63

            // Store the last Wasm SP into the `last_wasm_entry_sp` in the limits, if this
            // was core Wasm, otherwise store an invalid sentinal value.
            mov ", wasmfx_scratch1!(), ", -1
            cmove ", wasmfx_scratch1!(), ", rsp
            mov 40[", wasmfx_scratch0!(), "], ", wasmfx_scratch1!(), "

            // Tail call to the callee function pointer in the vmctx.
            jmp 16[", wasmfx_arg1!(), "]

            .cfi_endproc
        ",
            ),
        );

        #[cfg(test)]
        mod host_to_wasm_trampoline_offsets_tests {
            use wasmtime_environ::{Module, PtrSize, VMOffsets};

            #[test]
            fn test() {
                let module = Module::new();
                let offsets = VMOffsets::new(std::mem::size_of::<*mut u8>() as u8, &module);

                assert_eq!(8, offsets.vmctx_runtime_limits());
                assert_eq!(40, offsets.ptr.vmruntime_limits_last_wasm_entry_sp());
                assert_eq!(16, offsets.vmctx_callee());
                assert_eq!(0x65726f63, u32::from_le_bytes(*b"core"));
            }
        }
    } else {
        compile_error!("unsupported architecture");
    }
}

/// TODO
#[inline(always)]
pub fn cont_new(instance: &mut Instance, func: *mut u8) -> *mut u8 {
    let func = func as *mut VMFuncRef;
    let fnptr = unsafe { (*func).native_call.as_ptr() as *const VMFunctionBody };
    let vmctx1 = unsafe { (*func).vmctx };
    let vmctx2 = instance.vmctx();
    let fiber = Box::new(
        Fiber::new(
            FiberStack::new(4096).unwrap(),
            move |first_val: (), _suspend: &Suspend<_, u32, _>| {
                let trampoline = unsafe {
                    std::mem::transmute::<
                        *const VMFunctionBody,
                        unsafe extern "C" fn(*mut VMOpaqueContext, *mut VMContext, ()),
                    >(fnptr)
                };
                let trampoline = unsafe { prepare_host_to_wasm_trampoline(vmctx2, trampoline) };
                unsafe { trampoline(vmctx1, vmctx2, first_val) }
            },
        )
        .unwrap(),
    );
    let ptr: *mut Fiber<'static, (), u32, ()> = Box::into_raw(fiber);
    ptr as *mut _
}

/// TODO
#[inline(always)]
pub fn resume(instance: &mut Instance, cont: *mut u8) -> Result<u32, TrapReason> {
    let cont = cont as *mut Fiber<'static, (), u32, ()>;
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
        Ok(_) => {
            let drop_box: Box<Fiber<_, _, _>> = unsafe { Box::from_raw(cont) };
            drop(drop_box); // I think this would be covered by the close brace below anyway
            Ok(9999)
        }
        Err(y) => Ok(y),
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
