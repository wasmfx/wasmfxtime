//! This module contains a modified version of the `wasmtime_fiber` crate,
//! specialized for executing WasmFX continuations.

#![allow(missing_docs)]

use std::cell::Cell;
use std::io;
use std::ops::Range;
use wasmtime_continuations::{SwitchDirection, SwitchDirectionEnum, TagId};

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub mod unix;
        use unix as imp;
    } else {
        compile_error!("fibers are not supported on this platform");
    }
}

/// Represents an execution stack to use for a fiber.
#[derive(Debug)]
pub struct FiberStack(imp::FiberStack);

impl FiberStack {
    /// Creates a new fiber stack of the given size.
    pub fn new(size: usize) -> io::Result<Self> {
        Ok(Self(imp::FiberStack::new(size)?))
    }

    /// Creates a new fiber stack of the given size (using malloc).
    pub fn malloc(size: usize) -> io::Result<Self> {
        Ok(Self(imp::FiberStack::malloc(size)?))
    }

    /// Creates a new fiber stack with the given pointer to the bottom of the
    /// stack plus the byte length of the stack.
    ///
    /// The `bottom` pointer should be addressable for `len` bytes. The page
    /// beneath `bottom` should be unmapped as a guard page.
    ///
    /// # Safety
    ///
    /// This is unsafe because there is no validation of the given pointer.
    ///
    /// The caller must properly allocate the stack space with a guard page and
    /// make the pages accessible for correct behavior.
    pub unsafe fn from_raw_parts(bottom: *mut u8, len: usize) -> io::Result<Self> {
        Ok(Self(imp::FiberStack::from_raw_parts(bottom, len)?))
    }

    /// Gets the top of the stack.
    ///
    /// Returns `None` if the platform does not support getting the top of the
    /// stack.
    pub fn top(&self) -> Option<*mut u8> {
        self.0.top()
    }

    /// Returns the range of where this stack resides in memory if the platform
    /// supports it.
    pub fn range(&self) -> Option<Range<usize>> {
        self.0.range()
    }
}

pub struct Fiber {
    stack: FiberStack,
    inner: imp::Fiber,
    done: Cell<bool>,
}

pub struct Suspend {
    inner: imp::Suspend,
}

impl Fiber {
    /// Creates a new fiber which will execute `func` on the given stack.
    ///
    /// This function returns a `Fiber` which, when resumed, will execute `func`
    /// to completion. When desired the `func` can suspend itself via
    /// `Fiber::suspend`.
    pub fn new(stack: FiberStack, func: impl FnOnce((), &Suspend)) -> io::Result<Self> {
        let inner = imp::Fiber::new(&stack.0, func)?;

        Ok(Self {
            stack,
            inner,
            done: Cell::new(false),
        })
    }

    /// Resumes execution of this fiber.
    ///
    /// This function will transfer execution to the fiber and resume from where
    /// it last left off.
    ///
    /// Returns `true` if the fiber finished or `false` if the fiber was
    /// suspended in the middle of execution.
    ///
    /// # Panics
    ///
    /// Panics if the current thread is already executing a fiber or if this
    /// fiber has already finished.
    ///
    /// Note that if the fiber itself panics during execution then the panic
    /// will be propagated to this caller.
    pub fn resume(&self) -> SwitchDirection {
        assert!(!self.done.replace(true), "cannot resume a finished fiber");
        let reason = self.inner.resume(&self.stack.0);
        if let SwitchDirection {
            discriminant: SwitchDirectionEnum::Suspend,
            data: _,
        } = reason
        {
            self.done.set(false)
        };
        reason
    }

    /// Returns whether this fiber has finished executing.
    pub fn done(&self) -> bool {
        self.done.get()
    }

    /// Gets the stack associated with this fiber.
    pub fn stack(&self) -> &FiberStack {
        &self.stack
    }
}

impl Suspend {
    /// Suspend execution of a currently running fiber.
    ///
    /// This function will switch control back to the original caller of
    /// `Fiber::resume`. This function will then return once the `Fiber::resume`
    /// function is called again.
    ///
    /// # Panics
    ///
    /// Panics if the current thread is not executing a fiber from this library.
    pub fn suspend(&self, tag: TagId) {
        let reason = SwitchDirection::suspend(tag);
        self.inner.switch(reason);
    }

    fn execute(inner: imp::Suspend, func: impl FnOnce((), &Suspend)) {
        let suspend = Suspend { inner };
        // Note that the original wasmtime-fiber crate runs `func` wrapped in
        // `panic::catch_unwind`, to stop panics from being propagated onward,
        // instead just reporting parent. We eschew this, doing nothing special
        // about panics. This is justified because we only ever call this
        // function such that `func` is a closure around a call to a
        // `VMArrayCallFunction`, namely a host-to-wasm trampoline. It is thus
        // guaranteed not to panic.
        (func)((), &suspend);
        let reason = SwitchDirection::return_();
        suspend.inner.switch(reason);
    }
}

impl Drop for Fiber {
    fn drop(&mut self) {
        debug_assert!(self.done.get(), "fiber dropped without finishing");
    }
}
