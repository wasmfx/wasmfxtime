use crate::traphandlers::tls;
use crate::VMContext;
use std::mem;

pub use crate::sys::capi::{self, wasmtime_longjmp};

#[allow(missing_docs)]
pub type SignalHandler<'a> = dyn Fn() + Send + Sync + 'a;

pub unsafe fn wasmtime_setjmp(
    jmp_buf: *mut *const u8,
    callback: extern "C" fn(*mut u8, *mut VMContext),
    payload: *mut u8,
    callee: *mut VMContext,
) -> i32 {
    let callback = mem::transmute::<
        extern "C" fn(*mut u8, *mut VMContext),
        extern "C" fn(*mut u8, *mut u8),
    >(callback);
    capi::wasmtime_setjmp(jmp_buf, callback, payload, callee.cast())
}

pub fn platform_init(_macos_use_mach_ports: bool) {
    unsafe {
        capi::wasmtime_init_traps(handle_trap);
    }
}

extern "C" fn handle_trap(ip: usize, fp: usize, has_faulting_addr: bool, faulting_addr: usize) {
    tls::with(|info| {
        let info = match info {
            Some(info) => info,
            None => return,
        };
        let faulting_addr = if has_faulting_addr {
            Some(faulting_addr)
        } else {
            None
        };
        let ip = ip as *const u8;
        let jmp_buf = info.take_jmp_buf_if_trap(ip, |_handler| {
            panic!("custom signal handlers are not supported on this platform");
        });
        if !jmp_buf.is_null() {
            info.set_jit_trap(ip, fp, faulting_addr);
            unsafe { wasmtime_longjmp(jmp_buf) }
        }
    })
}

pub fn lazy_per_thread_init() {}
