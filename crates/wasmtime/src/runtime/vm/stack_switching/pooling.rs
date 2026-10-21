//! Experimental pooling allocator for continuation stacks.
//!
//! It mirrors the existing async fiber-stack pool without coupling
//! continuation allocation to the instance allocator.

use super::VMContinuationStack;
use crate::config::PoolingAllocationConfig;
use crate::prelude::*;

cfg_select! {
    all(
        unix,
        not(miri),
        not(asan),
        any(target_arch = "x86_64", target_arch = "aarch64"),
    ) => {
        mod imp {
            use super::*;
            use crate::runtime::vm::{HostAlignedByteCount, Mmap, mmap::AlignedLength};

            /// A single reservation split into guarded continuation-stack slots.
            #[derive(Debug)]
            pub struct ContinuationStackPool {
                mapping: Mmap<AlignedLength>,
                stack_size: HostAlignedByteCount,
                max_stacks: usize,
                page_size: HostAlignedByteCount,
                next_stack: usize,
            }

            impl ContinuationStackPool {
                pub fn new(config: &PoolingAllocationConfig, stack_size: usize) -> Result<Self> {
                    use rustix::mm::{MprotectFlags, mprotect};

                    let page_size = HostAlignedByteCount::host_page_size();
                    let stack_size = if stack_size == 0 {
                        HostAlignedByteCount::ZERO
                    } else {
                        HostAlignedByteCount::new_rounded_up(stack_size)
                            .and_then(|size| size.checked_add(page_size))
                            .context("continuation stack size exceeds addressable memory")?
                    };
                    let max_stacks = usize::try_from(config.limits.total_stacks).unwrap();
                    let allocation_size = stack_size.checked_mul(max_stacks).context(
                        "total size of continuation stacks exceeds addressable memory",
                    )?;
                    let mapping = Mmap::accessible_reserved(allocation_size, allocation_size)
                        .context("failed to create continuation stack pool mapping")?;

                    if !allocation_size.is_zero() {
                        unsafe {
                            for index in 0..max_stacks {
                                let offset = stack_size.unchecked_mul(index);
                                let guard = mapping
                                    .as_ptr()
                                    .add(offset.byte_count())
                                    .cast_mut();
                                mprotect(
                                    guard.cast(),
                                    page_size.byte_count(),
                                    MprotectFlags::empty(),
                                )
                                .context("failed to protect continuation stack guard page")?;
                            }
                        }
                    }

                    Ok(Self {
                        mapping,
                        stack_size,
                        max_stacks,
                        page_size,
                        next_stack: 0,
                    })
                }

                pub fn allocate(&mut self) -> Result<VMContinuationStack> {
                    if self.stack_size.is_zero() {
                        bail!(
                            "pooling allocator not configured to enable continuation stack allocation"
                        );
                    }
                    if self.next_stack == self.max_stacks {
                        bail!(
                            "maximum concurrent continuation stack limit of {} reached",
                            self.max_stacks
                        );
                    }

                    let index = self.next_stack;
                    self.next_stack += 1;

                    unsafe {
                        let usable_size = self
                            .stack_size
                            .checked_sub(self.page_size)
                            .expect("a non-empty stack slot includes its guard page");
                        let slot = self.stack_size.unchecked_mul(index);
                        let bottom = self
                            .mapping
                            .as_ptr()
                            .add(slot.byte_count() + self.page_size.byte_count())
                            .cast_mut();

                        match VMContinuationStack::from_raw_parts(
                            bottom,
                            self.page_size.byte_count(),
                            usable_size.byte_count(),
                        ) {
                            Ok(stack) => Ok(stack),
                            Err(error) => {
                                self.next_stack -= 1;
                                Err(error)
                            }
                        }
                    }
                }
            }
        }
    }
    _ => {
        mod imp {
            use super::*;

            /// Fallback that enforces the pooling limit while using
            /// the native continuation-stack allocation mechanism.
            #[derive(Debug)]
            pub struct ContinuationStackPool {
                stack_size: usize,
                max_stacks: usize,
                next_stack: usize,
            }

            impl ContinuationStackPool {
                pub fn new(config: &PoolingAllocationConfig, stack_size: usize) -> Result<Self> {
                    Ok(Self {
                        stack_size,
                        max_stacks: usize::try_from(config.limits.total_stacks).unwrap(),
                        next_stack: 0,
                    })
                }

                pub fn allocate(&mut self) -> Result<VMContinuationStack> {
                    if self.next_stack == self.max_stacks {
                        bail!(
                            "maximum concurrent continuation stack limit of {} reached",
                            self.max_stacks
                        );
                    }

                    self.next_stack += 1;
                    match VMContinuationStack::new(self.stack_size) {
                        Ok(stack) => Ok(stack),
                        Err(error) => {
                            self.next_stack -= 1;
                            Err(error)
                        }
                    }
                }
            }
        }
    }
}

pub use imp::ContinuationStackPool;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::InstanceLimits;

    #[test]
    fn continuation_stack_pool_honors_zero_limit() -> Result<()> {
        let config = PoolingAllocationConfig {
            limits: InstanceLimits {
                total_stacks: 0,
                ..Default::default()
            },
            ..PoolingAllocationConfig::default()
        };
        let mut pool = ContinuationStackPool::new(&config, 1)?;

        assert!(pool.allocate().is_err());
        Ok(())
    }

    #[cfg(all(unix, not(miri), not(asan), target_arch = "x86_64"))]
    #[test]
    fn continuation_stack_pool_enforces_its_limit() -> Result<()> {
        let config = PoolingAllocationConfig {
            limits: InstanceLimits {
                total_stacks: 2,
                ..Default::default()
            },
            ..PoolingAllocationConfig::default()
        };
        let mut pool = ContinuationStackPool::new(&config, 1)?;

        let first = pool.allocate()?;
        let second = pool.allocate()?;
        assert!(first.is_from_raw_parts());
        assert!(second.is_from_raw_parts());
        assert_ne!(first.range(), second.range());
        assert!(pool.allocate().is_err());
        Ok(())
    }
}
