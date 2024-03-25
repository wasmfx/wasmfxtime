//! The stack layout is expected to look like so:
//!
//!
//! ```text
//! 0xB000 +-----------------------+   <- top of stack (TOS)
//!        | *const u8             |   <- "dummy return PC"
//! 0xAff8 +-----------------------+
//!        | *const u8             |   <- "resume frame pointer"
//! 0xAff0 +-----------------------+   <- 16-byte aligned
//!        |                       |
//!        ~        ...            ~   <- actual native stack space to use
//!        |                       |
//! 0x1000 +-----------------------+
//!        |  guard page           |   <- (not currently enabled)
//! 0x0000 +-----------------------+
//! ```
//!
//! The meaning of the first two values are as follows:
//!
//! 1. Resume frame pointer (at TOS - 0x10, 0xAff0 above):
//!
//! This value indicates how to resume computation.
//! We  distinguish two cases
//!
//! 1.1
//! If the continuation is currently active (i.e., running directly, or ancestor
//! of the running continuation), it points into the stack of the parent of the
//! continuation, which looks something like this: Here, we assume that some
//! funtion $g resume-d the active continuation.
//!
//! //! ```text
//!
//! 0xF000 +-----------------------+
//!        |return PC ($g's caller)|   <- beginning of $g's frame
//! 0xEFF8 | - - - - - - - - - - - |
//!        |frame ptr ($g's caller)|
//! 0xEFF0 | - - - - - - - - - - - |
//!        ~         ...           ~
//!        |   stack frame of wasm |
//!  ...   |       function $g     |
//!        |     calling resume    |
//!        ~ - - - - - - - - - - - ~
//!        |  caller-saved regs    |
//!        |    stored here        |
//! 0xD000 +-----------------------+   <- beginning of pseudo frame
//!        |    return PC of $g    |      of wasmtime_fibre_switch
//! 0xCFF8 | - - - - - - - - - - - |
//!        |   saved RBP (of $g)   |   <- "pseudo frame pointer" of
//! 0xCFF0 | - - - - - - - - - - - |      wasmtime_fibre_switch
//!        |   saved RBX (of $g)   |
//! 0xCFE8 | - - - - - - - - - - - |
//!        |   saved R12 (of $g)   |
//! 0xCFE0 | - - - - - - - - - - - |
//!        |   saved R13 (of $g)   |
//! 0xCFD8 | - - - - - - - - - - - |
//!        |   saved R14 (of $g)   |
//! 0xCFD0 | - - - - - - - - - - - |
//!        |   saved R15 (of $g)   |
//! 0xCFC8 +-----------------------+ <- stack pointer at time of
//!                                         switching away
//! ```
//! Here, the pseudo-frame of the wasmtime_fibre_switch invocation that switched
//! to the active continuation begins at 0xD000. It's only a pseudo-frame
//! in the sense that we never go back to it by executing a ret instruction, but by
//! switching back to it using another invocation of wasmtime_fibre_switch. The
//! "resume frame pointer" stored in the active continuation (i.e., at 0xAff0 in
//! the first picture) is then the "pseudo frame pointer" of wamtime_fibre_switch.
//! In other words, we store 0xCFF0 at 0xAFF0.
//!
//! 1.2
//! If the first picture shows a suspended computation, then we also store a
//! "pseudo frame pointer" of wamtime_fibre_switch at TOS - 0x10, but this time
//! the one that resulted from calling wasmtime_fibre_switch when suspending.
//! (i.e., the stored pseudo frame pointer resides within the continuation's own
//! stack).
//!
//!
//! 2. Dummy return PC (at TOS - 0x10, 0xAff0 above):
//! The goal of the layout described in the previous two pictures is to ensure
//! the following: Whenever a continuation is active, the values at TOS - 0x08
//! and TOS - 0x10 together look like the beginning of an ordinary stack frame:
//! Address TOS - 0x10 (called 0xAff0 in first picutre) denotes its frame
//! pointer, and in turn contains the frame pointer of its "caller". Here, the
//! "caller" is supposed to be the parent continuation, or rather the call to
//! `wasmtime_fibre_switch` from the parent. In order to make sure that things
//! indeed look like a valid stack, we need to put a return PC above the frame
//! pointer. Thus, at TOS - 0x08 (called 0xAff8 in first picture), we store a PC
//! that's inside wasmtime_fibre_switch. Note that this PC is never used to
//! execute an actual ret instruction, but it ensures that any external tool
//! walking the frame pointer chain to construct a backtrace sees that the
//! "calling" function is wasmtime_fibre_switch, and the latter's caller is the
//! function that invoked `resume`.
//!
//! Note that this design ensures that external tools can construct backtraces
//! in the presence of stack switching by using frame pointers only. Wasmtime's
//! own mechanism for constructing back traces also relies on frame pointer
//! chains. However, it understands continuations and does not rely on the
//! trickery outlined here to go from the frames in one continuation to the
//! parent.

#![allow(unused_macros)]

use std::alloc::{alloc, dealloc, Layout};
use std::io;
use std::ops::Range;
use std::ptr;
use wasmtime_continuations::SwitchDirection;

#[derive(Debug)]
pub struct FiberStack {
    // The top of the stack; for stacks allocated by the fiber implementation itself,
    // the base address of the allocation will be `top.sub(len.unwrap())`
    top: *mut u8,
    // The length of the stack
    len: usize,
    // whether or not this stack was mmap'd
    mmap: bool,
}

impl FiberStack {
    pub fn new(size: usize) -> io::Result<Self> {
        // Round up our stack size request to the nearest multiple of the
        // page size.
        let page_size = rustix::param::page_size();
        let size = if size == 0 {
            page_size
        } else {
            (size + (page_size - 1)) & (!(page_size - 1))
        };

        unsafe {
            // Add in one page for a guard page and then ask for some memory.
            let mmap_len = size + page_size;
            let mmap = rustix::mm::mmap_anonymous(
                ptr::null_mut(),
                mmap_len,
                rustix::mm::ProtFlags::empty(),
                rustix::mm::MapFlags::PRIVATE,
            )?;

            rustix::mm::mprotect(
                mmap.cast::<u8>().add(page_size).cast(),
                size,
                rustix::mm::MprotectFlags::READ | rustix::mm::MprotectFlags::WRITE,
            )?;

            Ok(Self {
                top: mmap.cast::<u8>().add(mmap_len),
                len: mmap_len,
                mmap: true,
            })
        }
    }

    pub fn malloc(size: usize) -> io::Result<Self> {
        unsafe {
            let layout = Layout::array::<u8>(size).unwrap();
            let base = alloc(layout);
            FiberStack::from_raw_parts(base, size)
        }
    }

    #[allow(clippy::missing_safety_doc)]
    pub unsafe fn from_raw_parts(base: *mut u8, len: usize) -> io::Result<Self> {
        Ok(Self {
            top: base.add(len),
            len,
            mmap: false,
        })
    }

    pub fn top(&self) -> Option<*mut u8> {
        Some(self.top)
    }

    pub fn range(&self) -> Option<Range<usize>> {
        let base = unsafe { self.top.sub(self.len) as usize };
        Some(base..base + self.len)
    }
}

impl Drop for FiberStack {
    fn drop(&mut self) {
        unsafe {
            if self.mmap {
                let ret = rustix::mm::munmap(self.top.sub(self.len) as _, self.len);
                debug_assert!(ret.is_ok());
            } else {
                let layout = Layout::array::<u8>(self.len).unwrap();
                dealloc(self.top.sub(self.len), layout);
            }
        }
    }
}

pub struct Fiber;

pub struct Suspend(*mut u8);

extern "C" {
    fn wasmtime_fibre_init(
        top_of_stack: *mut u8,
        entry: extern "C" fn(*mut u8, *mut u8),
        entry_arg0: *mut u8,
        wasmtime_fibre_switch: *const u8,
    );
    fn wasmtime_fibre_switch(top_of_stack: *mut u8, payload: u64) -> u64;
    #[allow(dead_code)] // only used in inline assembly for some platforms
    fn wasmtime_fibre_start();
}

extern "C" fn fiber_start<F>(arg0: *mut u8, top_of_stack: *mut u8)
where
    F: FnOnce((), &super::Suspend),
{
    unsafe {
        let inner = Suspend(top_of_stack);
        super::Suspend::execute(inner, Box::from_raw(arg0.cast::<F>()))
    }
}

impl Fiber {
    pub fn new<F>(stack: &FiberStack, func: F) -> io::Result<Self>
    where
        F: FnOnce((), &super::Suspend),
    {
        unsafe {
            let data = Box::into_raw(Box::new(func)).cast();
            wasmtime_fibre_init(
                stack.top,
                fiber_start::<F>,
                data,
                wasmtime_fibre_switch as *const u8,
            );
        }

        Ok(Self)
    }

    pub(crate) fn resume(&self, stack: &FiberStack) -> SwitchDirection {
        unsafe {
            let reason = SwitchDirection::resume().into();
            SwitchDirection::from(wasmtime_fibre_switch(stack.top, reason))
        }
    }
}

impl Suspend {
    pub fn switch(&self, payload: SwitchDirection) {
        unsafe {
            let arg = payload.into();
            wasmtime_fibre_switch(self.0, arg);
        }
    }

    pub fn from_top_ptr(ptr: *mut u8) -> Self {
        Suspend(ptr)
    }
}

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        mod x86_64;
    } else {
        compile_error!("fibers are not supported on this CPU architecture");
    }
}
