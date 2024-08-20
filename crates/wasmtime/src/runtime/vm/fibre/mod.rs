//! This module contains a modified version of the `wasmtime_fiber` crate,
//! specialized for executing WasmFX continuations.

#![allow(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(not(feature = "wasmfx_baseline"))] {

        use std::cell::Cell;
        use std::io;
        use std::ops::Range;
        use wasmtime_continuations::ControlEffect;

        use crate::runtime::vm::{VMContext, VMFuncRef, ValRaw};

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

        impl Fiber {
            /// Creates a new fiber which will execute `func` on the given stack.
            ///
            /// This function returns a `Fiber` which, when resumed, will execute `func`
            /// to completion. When desired the `func` can suspend itself via
            /// `Fiber::suspend`.
            pub fn new(
                stack: FiberStack,
                func_ref: *const VMFuncRef,
                caller_vmctx: *mut VMContext,
                args_ptr: *mut ValRaw,
                args_capacity: usize,
            ) -> io::Result<Self> {
                let inner = imp::Fiber::new(&stack.0, func_ref, caller_vmctx, args_ptr, args_capacity)?;

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
            pub fn resume(&self) -> ControlEffect {
                assert!(!self.done.replace(true), "cannot resume a finished fiber");
                let reason = self.inner.resume(&self.stack.0);
                if ControlEffect::is_suspend(reason) {
                    self.done.set(false)
                }
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

        impl Drop for Fiber {
            fn drop(&mut self) {
                debug_assert!(self.done.get(), "fiber dropped without finishing");
            }
        }
    }
}
