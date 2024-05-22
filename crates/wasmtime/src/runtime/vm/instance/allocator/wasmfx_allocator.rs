// WasmFX fiber stack allocators
//
// * on_demand: allocates memory lazily
// * pooling: preallocates a chunk of memory eagerly
//

use anyhow::Result;
use wasmtime_continuations::WasmFXConfig;

cfg_if::cfg_if! {
    if #[cfg(feature = "wasmfx_baseline")] {
        use wasmtime_fiber as fiber_impl;
    } else {
        use crate::vm::fibre as fiber_impl;
    }
}

pub type FiberStack = fiber_impl::FiberStack;

#[cfg(not(feature = "wasmfx_pooling_allocator"))]
pub mod wasmfx_on_demand {
    use super::*;

    #[derive(Debug)]
    pub struct InnerAllocator {
        stack_size: usize,
    }

    impl InnerAllocator {
        pub fn new(config: &WasmFXConfig) -> Result<Self> {
            Ok(InnerAllocator {
                stack_size: config.stack_size,
            })
        }

        pub fn allocate(&self) -> Result<FiberStack> {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "unsafe_wasmfx_stacks", not(feature = "wasmfx_baseline")))] {
                    Ok(fiber_impl::FiberStack::malloc(self.stack_size).ok_or_err(|_| anyhow::anyhow!("Fiber stack allocation failed"))?)
                } else {
                    Ok(fiber_impl::FiberStack::new(self.stack_size).map_err(|_| anyhow::anyhow!("Fiber stack allocation failed"))?)
                }
            }
        }

        pub fn deallocate(&self, _stack: &FiberStack) {
            // The on-demand allocator has no further bookkeeping for fiber stacks
        }
    }
}

#[cfg(feature = "wasmfx_pooling_allocator")]
pub mod wasmfx_pooling {
    use super::*;

    use crate::runtime::vm::instance::allocator::pooling::{
        index_allocator::{SimpleIndexAllocator, SlotId},
        round_up_to_pow2,
    };
    use crate::runtime::vm::sys::vm::{commit_pages, decommit_pages};
    use crate::vm::Mmap;
    use anyhow::{anyhow, bail, Context, Result};

    /// Represents a pool of execution stacks (used for the async fiber implementation).
    ///
    /// Each index into the pool represents a single execution stack. The maximum number of
    /// stacks is the same as the maximum number of instances.
    ///
    /// As stacks grow downwards, each stack starts (lowest address) with a guard page
    /// that can be used to detect stack overflow.
    ///
    /// The top of the stack (starting stack pointer) is returned when a stack is allocated
    /// from the pool.
    #[derive(Debug)]
    pub struct InnerAllocator {
        mapping: Mmap,
        stack_size: usize,
        max_stacks: usize,
        page_size: usize,
        index_allocator: SimpleIndexAllocator,
        async_stack_zeroing: bool,
        async_stack_keep_resident: usize,
    }

    impl InnerAllocator {
        pub fn new(config: &WasmFXConfig) -> Result<Self> {
            use rustix::mm::{mprotect, MprotectFlags};

            let total_stacks = 1024 /* total amount of stacks */;

            let page_size = crate::vm::page_size();

            // Add a page to the stack size for the guard page when using fiber stacks
            let stack_size = if config.stack_size == 0 {
                0
            } else {
                round_up_to_pow2(config.stack_size, page_size)
                    .checked_add(page_size)
                    .ok_or_else(|| anyhow!("stack size exceeds addressable memory"))?
            };

            let max_stacks = usize::try_from(total_stacks).unwrap();

            let allocation_size = stack_size.checked_mul(max_stacks).ok_or_else(|| {
                anyhow!("total size of execution stacks exceeds addressable memory")
            })?;

            let mapping = Mmap::accessible_reserved(allocation_size, allocation_size)
                .context("failed to create stack pool mapping")?;

            // Set up the stack guard pages.
            if allocation_size > 0 {
                unsafe {
                    for i in 0..max_stacks {
                        // Make the stack guard page inaccessible.
                        let bottom_of_stack = mapping.as_ptr().add(i * stack_size).cast_mut();
                        mprotect(bottom_of_stack.cast(), page_size, MprotectFlags::empty())
                            .context("failed to protect stack guard page")?;
                    }
                }
            }

            Ok(Self {
                mapping,
                stack_size,
                max_stacks,
                page_size,
                async_stack_zeroing: false,
                async_stack_keep_resident: 0,
                index_allocator: SimpleIndexAllocator::new(total_stacks),
            })
        }

        /// Allocate a new fiber.
        pub fn allocate(&self) -> Result<FiberStack> {
            if self.stack_size == 0 {
                bail!("pooling allocator not configured to enable fiber stack allocation");
            }

            let index = self
                .index_allocator
                .alloc()
                .ok_or_else(|| {
                    anyhow!(
                        "maximum concurrent fiber limit of {} reached",
                        self.max_stacks
                    )
                })?
                .index();

            assert!(index < self.max_stacks);

            unsafe {
                // Remove the guard page from the size
                let size_without_guard = self.stack_size - self.page_size;

                let bottom_of_stack = self
                    .mapping
                    .as_ptr()
                    .add((index * self.stack_size) + self.page_size)
                    .cast_mut();

                commit_pages(bottom_of_stack, size_without_guard)?;

                let stack = super::FiberStack::from_raw_parts(bottom_of_stack, size_without_guard)?;
                Ok(stack)
            }
        }

        /// Deallocate a previously-allocated fiber.
        ///
        /// # Safety
        ///
        /// The fiber must have been allocated by this pool, must be in an allocated
        /// state, and must never be used again.
        pub unsafe fn deallocate(&self, stack: &FiberStack) {
            let top = stack
                .top()
                .expect("fiber stack not allocated from the pool") as usize;

            let base = self.mapping.as_ptr() as usize;
            let len = self.mapping.len();
            assert!(
                top > base && top <= (base + len),
                "fiber stack top pointer not in range"
            );

            // Remove the guard page from the size
            let stack_size = self.stack_size - self.page_size;
            let bottom_of_stack = top - stack_size;
            let start_of_stack = bottom_of_stack - self.page_size;
            assert!(start_of_stack >= base && start_of_stack < (base + len));
            assert!((start_of_stack - base) % self.stack_size == 0);

            let index = (start_of_stack - base) / self.stack_size;
            assert!(index < self.max_stacks);

            if self.async_stack_zeroing {
                self.zero_stack(bottom_of_stack, stack_size);
            }

            self.index_allocator.free(SlotId(index as u32));
        }

        fn zero_stack(&self, bottom: usize, size: usize) {
            // Manually zero the top of the stack to keep the pages resident in
            // memory and avoid future page faults. Use the system to deallocate
            // pages past this. This hopefully strikes a reasonable balance between:
            //
            // * memset for the whole range is probably expensive
            // * madvise for the whole range incurs expensive future page faults
            // * most threads probably don't use most of the stack anyway
            let size_to_memset = size.min(self.async_stack_keep_resident);
            unsafe {
                core::ptr::write_bytes(
                    (bottom + size - size_to_memset) as *mut u8,
                    0,
                    size_to_memset,
                );

                // Use the system to reset remaining stack pages to zero.
                decommit_pages(bottom as _, size - size_to_memset).unwrap();
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "wasmfx_pooling_allocator")] {
        use wasmfx_pooling as imp;
    } else {
        use wasmfx_on_demand as imp;
    }
}

#[derive(Debug)]
pub struct WasmFXAllocator {
    inner: imp::InnerAllocator,
}

impl WasmFXAllocator {
    pub fn new(config: &WasmFXConfig) -> Result<Self> {
        Ok(Self {
            inner: imp::InnerAllocator::new(config)?,
        })
    }

    pub fn allocate(&self) -> Result<FiberStack> {
        self.inner.allocate()
    }

    pub fn deallocate(&self, stack: &FiberStack) {
        cfg_if::cfg_if! {
            if #[cfg(feature = "wasmfx_pooling_allocator")] {
                unsafe { self.inner.deallocate(stack) }
            } else {
                self.inner.deallocate(stack)
            }
        }
    }
}
