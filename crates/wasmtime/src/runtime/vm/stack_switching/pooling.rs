//! Experimental pooling allocator for continuation stacks.
//!
//! It mirrors the existing async fiber-stack pool without coupling
//! continuation allocation to the instance allocator.

use super::VMContinuationStack;
use crate::config::PoolingAllocationConfig;
use crate::prelude::*;
use crate::runtime::vm::{SendSyncPtr, VMContRef, VMStackState};

/// Owners of the fixed stack slots in a continuation stack pool.
///
/// The store keeps every `VMContRef` alive longer than its stack pool, so these
/// pointers remain valid for the lifetime of this list. This list is bounded by
/// the configured maximum number of stacks.
#[derive(Debug)]
struct StackOwners {
    owners: TryVec<SendSyncPtr<VMContRef>>,
    next_recycle: usize,
}

impl StackOwners {
    fn with_capacity(capacity: usize) -> Result<Self> {
        Ok(Self {
            owners: TryVec::with_capacity(capacity)?,
            next_recycle: 0,
        })
    }

    fn len(&self) -> usize {
        self.owners.len()
    }

    fn push(&mut self, owner: SendSyncPtr<VMContRef>) -> Result<()> {
        self.owners.push(owner)?;
        Ok(())
    }

    fn recycle(&mut self) -> Option<SendSyncPtr<VMContRef>> {
        for offset in 0..self.owners.len() {
            let index = (self.next_recycle + offset) % self.owners.len();
            let mut owner = self.owners[index];
            // SAFETY: `StackOwners` only contains pointers to continuations
            // retained by the store that owns this pool. Allocation has
            // exclusive access to that store, and terminal continuations are
            // no longer executing on their stacks. The object is reset by the
            // store before its pointer is returned to Wasm again.
            let continuation = unsafe { owner.as_mut() };
            if matches!(
                continuation.common_stack_information.state,
                VMStackState::Returned | VMStackState::Trapped
            ) {
                self.next_recycle = (index + 1) % self.owners.len();
                return Some(owner);
            }
        }
        None
    }
}

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
                owners: StackOwners,
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
                        owners: StackOwners::with_capacity(max_stacks)?,
                    })
                }

                pub fn is_full(&self) -> bool {
                    self.owners.len() == self.max_stacks
                }

                pub fn recycle(&mut self) -> Option<SendSyncPtr<VMContRef>> {
                    debug_assert!(self.is_full());
                    self.owners.recycle()
                }

                pub fn allocate(
                    &mut self,
                    owner: SendSyncPtr<VMContRef>,
                ) -> Result<VMContinuationStack> {
                    if self.stack_size.is_zero() {
                        bail!(
                            "pooling allocator not configured to enable continuation stack allocation"
                        );
                    }
                    assert!(!self.is_full());

                    let index = self.owners.len();

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
                            Ok(stack) => {
                                self.owners.push(owner)?;
                                Ok(stack)
                            }
                            Err(error) => Err(error),
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
                owners: StackOwners,
            }

            impl ContinuationStackPool {
                pub fn new(config: &PoolingAllocationConfig, stack_size: usize) -> Result<Self> {
                    let max_stacks = usize::try_from(config.limits.total_stacks).unwrap();
                    Ok(Self {
                        stack_size,
                        max_stacks,
                        owners: StackOwners::with_capacity(max_stacks)?,
                    })
                }

                pub fn is_full(&self) -> bool {
                    self.owners.len() == self.max_stacks
                }

                pub fn recycle(&mut self) -> Option<SendSyncPtr<VMContRef>> {
                    debug_assert!(self.is_full());
                    self.owners.recycle()
                }

                pub fn allocate(
                    &mut self,
                    owner: SendSyncPtr<VMContRef>,
                ) -> Result<VMContinuationStack> {
                    assert!(!self.is_full());

                    match VMContinuationStack::new(self.stack_size) {
                        Ok(stack) => {
                            self.owners.push(owner)?;
                            Ok(stack)
                        }
                        Err(error) => Err(error),
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

    fn owner(continuation: &mut Box<VMContRef>) -> SendSyncPtr<VMContRef> {
        SendSyncPtr::from(core::ptr::NonNull::from(continuation.as_mut()))
    }

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

        assert!(pool.is_full());
        assert!(pool.recycle().is_none());
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
        let mut first_owner = Box::new(VMContRef::empty());
        let mut second_owner = Box::new(VMContRef::empty());

        let first = pool.allocate(owner(&mut first_owner))?;
        let second = pool.allocate(owner(&mut second_owner))?;
        assert!(first.is_from_raw_parts());
        assert!(second.is_from_raw_parts());
        assert_ne!(first.range(), second.range());
        let first_range = first.range();
        first_owner.stack = first;
        second_owner.stack = second;

        assert!(pool.is_full());
        assert!(pool.recycle().is_none());

        first_owner.common_stack_information.state = VMStackState::Returned;
        let recycled = pool.recycle().unwrap();
        assert_eq!(recycled.as_ptr(), first_owner.as_mut() as *mut VMContRef);
        assert_eq!(first_owner.stack.range(), first_range);

        first_owner.common_stack_information.state = VMStackState::Running;
        second_owner.common_stack_information.state = VMStackState::Trapped;
        let recycled = pool.recycle().unwrap();
        assert_eq!(recycled.as_ptr(), second_owner.as_mut() as *mut VMContRef);
        Ok(())
    }
}
