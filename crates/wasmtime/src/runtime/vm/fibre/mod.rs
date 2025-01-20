//! This module contains a modified version of the `wasmtime_fiber` crate,
//! specialized for executing WasmFX continuations.

#![allow(missing_docs)]

cfg_if::cfg_if! {
    if #[cfg(any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"))] {

        use std::io;
        use std::ops::Range;

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
        #[repr(C)]
        pub struct FiberStack(imp::FiberStack);

        impl FiberStack {
            /// Creates a new fiber stack of the given size.
            pub fn new(size: usize, _zeroed: bool /* TODO(dhil): unimplemented */) -> io::Result<Self> {
                Ok(Self(imp::FiberStack::new(size)?))
            }

            /// Returns a stack of size 0.
            pub fn unallocated() -> Self {
                Self(imp::FiberStack::unallocated())
            }

            /// Is this stack unallocated/of size 0?
            pub fn is_unallocated(&self) -> bool {
                imp::FiberStack::is_unallocated(&self.0)
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
            pub unsafe fn from_raw_parts(bottom: *mut u8, guard_size: usize, len: usize) -> io::Result<Self> {
                Ok(Self(imp::FiberStack::from_raw_parts(bottom, guard_size, len)?))
            }

            /// Is this a manually-managed stack created from raw parts? If so, it is up
            /// to whoever created it to manage the stack's memory allocation.
            pub fn is_from_raw_parts(&self) -> bool {
                self.0.is_from_raw_parts()
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

            /// Returns the instruction pointer stored in the Fiber's ControlContext.
            pub fn control_context_instruction_pointer(&self) -> usize {
                self.0.control_context_instruction_pointer()
            }

            /// Returns the frame pointer stored in the Fiber's ControlContext.
            pub fn control_context_frame_pointer(&self) -> usize {
                self.0.control_context_frame_pointer()
            }

            pub fn initialize(
                &self,
                func_ref: *const VMFuncRef,
                caller_vmctx: *mut VMContext,
                args_ptr: *mut ValRaw,
                args_capacity: usize,
            ) {
                self.0.initialize(func_ref, caller_vmctx, args_ptr, args_capacity)
            }
        }


    }
}
