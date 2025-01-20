// WasmFX fiber stack allocators
//
// * on_demand: allocates memory lazily
// * pooling: preallocates a chunk of memory eagerly
//

use crate::prelude::*;
use anyhow::Result;
use wasmtime_continuations::WasmFXConfig;

pub use crate::runtime::vm::continuation::imp::{FiberStack, VMContRef};

// This module is dead code if the pooling allocator is toggled.
#[allow(dead_code)]
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

        pub fn allocate(&mut self) -> Result<(*mut VMContRef, FiberStack)> {
            let stack = {
                cfg_if::cfg_if! {
                    if #[cfg(all(feature = "unsafe_wasmfx_stacks", any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline")))] {
                        super::FiberStack::malloc(self.stack_size)
                    } else {
                        super::FiberStack::new(self.stack_size, false /* whether zeroed */)
                    }
                }
            };
            let stack = stack.map_err(|_| anyhow::anyhow!("Fiber stack allocation failed"));
            let contref = Box::into_raw(Box::new(VMContRef::empty()));
            Ok((contref, stack?))
        }

        pub fn deallocate(&mut self, contref: *mut VMContRef) {
            // In on-demand mode, we actually deallocate the continuation.
            unsafe { core::mem::drop(Box::from_raw(contref)) };
        }
    }
}

// This module is dead code if the on-demand allocator is toggled.
#[allow(dead_code)]
// We want to compile this module unconditionally, even if the
// `wasmfx_pooling_allocator` feature is disabled. However, its implementation
// depends on code only available if Wasmtime's own `pooling-allocator` feature
// is enabled.
#[cfg(feature = "pooling-allocator")]
pub mod wasmfx_pooling {
    use super::*;

    use crate::runtime::vm::instance::allocator::pooling::index_allocator::{
        SimpleIndexAllocator, SlotId,
    };
    use crate::runtime::vm::sys::vm::commit_pages;
    use crate::vm::{mmap::AlignedLength, HostAlignedByteCount, Mmap};
    use anyhow::{anyhow, bail, Context, Result};

    /// Represents a pool of `VMContRef`s and their corresponding execution stacks.
    ///
    /// Each index into the pool represents a single pair of `VMContRef` and its
    /// corresponding execution stack. The maximum number of stacks is the same
    /// as the maximum number of instances.
    ///
    ///
    /// As stacks grow downwards, each stack starts (lowest address) with a guard page
    /// that can be used to detect stack overflow.
    ///
    /// The top of the stack (starting stack pointer) is returned when a stack is allocated
    /// from the pool.
    pub struct InnerAllocator {
        continuations: Vec<VMContRef>,
        stack_mapping: Mmap<AlignedLength>,
        stack_size: HostAlignedByteCount,
        max_stacks: usize,
        page_size: HostAlignedByteCount,
        index_allocator: SimpleIndexAllocator,
    }

    impl InnerAllocator {
        pub fn new(config: &WasmFXConfig) -> Result<Self> {
            use rustix::mm::{mprotect, MprotectFlags};

            let total_stacks : u32 = 1024 /* total amount of stacks */;

            let page_size = HostAlignedByteCount::host_page_size();

            // Add a page to the stack size for the guard page when using fiber stacks
            let stack_size = if config.stack_size == 0 {
                HostAlignedByteCount::ZERO
            } else {
                HostAlignedByteCount::new_rounded_up(config.stack_size)
                    .and_then(|size| size.checked_add(HostAlignedByteCount::host_page_size()))
                    .context("stack size exceeds addressable memory")?
            };

            let max_stacks = usize::try_from(total_stacks).unwrap();

            let allocation_size = stack_size
                .checked_mul(max_stacks)
                .context("total size of execution stacks exceeds addressable memory")?;

            let stack_mapping = Mmap::accessible_reserved(allocation_size, allocation_size)
                .context("failed to create stack pool mapping")?;

            // Set up the stack guard pages.
            if !allocation_size.is_zero() {
                unsafe {
                    for i in 0..max_stacks {
                        // Safety: i < max_stacks and we've already checked that
                        // stack_size * max_stacks is valid.
                        let offset = stack_size.unchecked_mul(i);
                        // Make the stack guard page inaccessible.
                        let bottom_of_stack =
                            stack_mapping.as_ptr().add(offset.byte_count()).cast_mut();
                        mprotect(
                            bottom_of_stack.cast(),
                            page_size.byte_count(),
                            MprotectFlags::empty(),
                        )
                        .context("failed to protect stack guard page")?;
                    }
                }
            }

            let mut continuations = Vec::with_capacity(total_stacks as usize);
            continuations.resize_with(total_stacks as usize, VMContRef::empty);

            Ok(Self {
                continuations,
                stack_mapping,
                stack_size,
                max_stacks,
                page_size,
                index_allocator: SimpleIndexAllocator::new(total_stacks),
            })
        }

        /// Allocate a new fiber.
        pub fn allocate(&mut self) -> Result<(*mut VMContRef, FiberStack)> {
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
                let size_without_guard = self.stack_size.byte_count() - self.page_size.byte_count();

                let bottom_of_stack = self
                    .stack_mapping
                    .as_ptr()
                    .add(self.stack_size.unchecked_mul(index).byte_count())
                    .cast_mut();

                commit_pages(bottom_of_stack, size_without_guard)?;

                let stack = super::FiberStack::from_raw_parts(
                    bottom_of_stack,
                    self.page_size.byte_count(),
                    size_without_guard,
                )?;
                let continuation = &mut self.continuations[index];
                Ok((continuation as *mut VMContRef, stack))
            }
        }

        /// Deallocate a previously-allocated fiber.
        ///
        /// # Safety
        ///
        /// The fiber must have been allocated by this pool, must be in an allocated
        /// state, and must never be used again.
        pub fn deallocate(&mut self, continuation: *mut VMContRef) {
            let continuation = unsafe { continuation.as_mut().unwrap() };

            // While in storage, the continuation only stores a dummy stack.
            let fiber_stack = continuation.detach_stack();

            // Let's make sure that the fiber_stack is indeed custom allocated,
            // so that it going out of scope here does not attempt to deallocate it
            debug_assert!(fiber_stack.is_from_raw_parts());

            let top = fiber_stack
                .top()
                .expect("fiber stack not allocated from the pool") as usize;

            let base = self.stack_mapping.as_ptr() as usize;
            let len = self.stack_mapping.len();
            assert!(
                top > base && top <= (base + len),
                "fiber stack top pointer not in range"
            );

            // Remove the guard page from the size
            let stack_size = self.stack_size.byte_count() - self.page_size.byte_count();
            let bottom_of_stack = top - stack_size;
            let start_of_stack =
                // TODO(dhil): The fiber and fibre
                // interfaces/implementations are slightly out of
                // sync; in one of them the page_size is part of the
                // stack size, in the other it isn't. We should bring
                // them into sync.
                if cfg!(feature = "wasmfx_baseline") && cfg!(not(feature = "wasmfx_no_baseline")) {
                    bottom_of_stack - self.page_size.byte_count()
                } else {
                    bottom_of_stack
                };
            assert!(start_of_stack >= base && start_of_stack < (base + len));
            assert!((start_of_stack - base) % self.stack_size.byte_count() == 0);

            let index = (start_of_stack - base) / self.stack_size.byte_count();
            assert!(index < self.max_stacks);

            // If the `FiberStack` has the given `index` in the pool, then the
            // `VMContRef` must also be at that index in the `continuations`
            // vector.
            assert_eq!(
                continuation as *mut VMContRef,
                &mut self.continuations[index] as *mut VMContRef
            );

            let index = u32::try_from(index).unwrap();
            self.index_allocator.free(SlotId(index));
        }
    }

    impl Drop for InnerAllocator {
        fn drop(&mut self) {
            cfg_if::cfg_if! {
                if #[cfg(all(feature = "wasmfx_baseline", not(feature = "wasmfx_no_baseline")))] {
                    // This is a workaround for the following quirk:
                    //
                    // We are about to drop all the `VMContRef`s in the
                    // `continuations` vector. However, if any of the continuations
                    // have not run to completion, dropping the corresponding
                    // `Fiber` will panic (at least in the baseline implementation).
                    // Since we are not currently enforcing that all
                    // continuations created with cont.new must be run to
                    // completion or cancelled in some other way, we must avoid
                    // those panics.
                    //
                    // To this end, we `forget` the Fiber instead of properly
                    // Drop-ping it. Since the corresponding `FiberStack` is
                    // custom allocated, its Drop implementation does nothing
                    // anyway, meaning that this does not leak memory.
                    for cont in self.continuations.drain(..) {
                        cont.fiber.map(|b| {
                            // Note that we consume the Box to get the `Fiber`,
                            // meaning that the Box itself doesn't leak.
                            core::mem::forget(*b);
                        });
                    }
                }
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(all(feature = "wasmfx_pooling_allocator"))] {
        use wasmfx_pooling as imp;
    } else {
        use wasmfx_on_demand as imp;
    }
}

pub struct WasmFXAllocator {
    inner: imp::InnerAllocator,
}

impl WasmFXAllocator {
    pub fn new(config: &WasmFXConfig) -> Result<Self> {
        Ok(Self {
            inner: imp::InnerAllocator::new(config)?,
        })
    }

    /// Note that for technical reasons, we return the `VMContRef` and
    /// `FiberStack` separately. In particular, the stack field of the
    /// continuation does not correspond to/point to that stack, yet. Instead, the
    /// `VMContRef` returned here has an empty stack (i.e., `None` in the
    /// baseline implementation, or an empty dummy stack in the optimized
    /// implementation).
    /// This allows the baseline implementation of the allocator interface to
    /// initialize a new `Fiber` from the `FiberStack`. then save it in the
    /// `VMContRef`.
    ///
    /// Note that the `revision` counter of the returned `VMContRef` may be
    /// non-zero and must not be decremented.
    pub fn allocate(&mut self) -> Result<(*mut VMContRef, FiberStack)> {
        self.inner.allocate()
    }

    /// This may not actually deallocate the underlying memory, but simply
    /// return the `VMContRef` to a pool.
    pub fn deallocate(&mut self, contref: *mut VMContRef) {
        self.inner.deallocate(contref)
    }
}
