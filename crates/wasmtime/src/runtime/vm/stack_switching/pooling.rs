//! Experimental pooling allocator for continuation stacks.
//!
//! It mirrors the existing async fiber-stack pool without coupling
//! continuation allocation to the instance allocator.

use super::VMContinuationStack;
use crate::config::PoolingAllocationConfig;
use crate::prelude::*;
use crate::runtime::vm::{SendSyncPtr, VMContRef, VMStackState};
use crate::sync::Mutex;
use std::sync::Arc;

use imp::StackSlot;

#[derive(Debug)]
struct StackOwner {
    continuation: SendSyncPtr<VMContRef>,
    slot: StackSlot,
}

/// Owners of the fixed stack slots in a continuation stack pool.
///
/// The store retains every `VMContRef` while allocation can inspect
/// this list, so these pointers remain valid whenever they are
/// dereferenced. During store destruction the continuations are
/// dropped first; this pool's destructor only reads the slot
/// identifiers and does not dereference the owner pointers.
#[derive(Debug)]
struct StackOwners {
    owners: TryVec<StackOwner>,
    next_recycle: usize,
}

impl StackOwners {
    fn with_capacity(capacity: usize) -> Result<Self> {
        Ok(Self {
            owners: TryVec::with_capacity(capacity)?,
            next_recycle: 0,
        })
    }

    fn push(&mut self, continuation: SendSyncPtr<VMContRef>, slot: StackSlot) -> Result<()> {
        self.owners.push(StackOwner { continuation, slot })?;
        Ok(())
    }

    fn recycle(&mut self) -> Option<SendSyncPtr<VMContRef>> {
        for offset in 0..self.owners.len() {
            let index = (self.next_recycle + offset) % self.owners.len();
            let mut owner = self.owners[index].continuation;
            // SAFETY: `StackOwners` only contains pointers to
            // continuations retained by the store that owns this
            // pool. Allocation has exclusive access to that store,
            // and terminal continuations are no longer executing on
            // their stacks. The object is reset by the store before
            // its pointer is returned to Wasm again.
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

    fn slots(&self) -> impl Iterator<Item = StackSlot> + '_ {
        self.owners.iter().map(|owner| owner.slot)
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

            #[derive(Clone, Copy, Debug)]
            pub(super) struct StackSlot(usize);

            #[derive(Debug)]
            struct ArenaState {
                next: usize,
                free: TryVec<StackSlot>,
            }

            /// Engine-owned reservation split into guarded continuation-stack slots.
            #[derive(Debug)]
            pub struct ContinuationStackArena {
                mapping: Mmap<AlignedLength>,
                stack_size: HostAlignedByteCount,
                max_stacks: usize,
                page_size: HostAlignedByteCount,
                state: Mutex<ArenaState>,
            }

            impl ContinuationStackArena {
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
                        state: Mutex::new(ArenaState {
                            next: 0,
                            free: TryVec::with_capacity(max_stacks)?,
                        }),
                    })
                }

                pub(super) fn allocate(
                    &self,
                ) -> Result<Option<(VMContinuationStack, StackSlot)>> {
                    if self.stack_size.is_zero() {
                        bail!(
                            "pooling allocator not configured to enable continuation stack allocation"
                        );
                    }

                    let mut state = self.state.lock();
                    let slot = match state.free.pop() {
                        Some(slot) => slot,
                        None if state.next < self.max_stacks => {
                            let slot = StackSlot(state.next);
                            state.next += 1;
                            slot
                        }
                        None => return Ok(None),
                    };

                    unsafe {
                        let usable_size = self
                            .stack_size
                            .checked_sub(self.page_size)
                            .expect("a non-empty stack slot includes its guard page");
                        let offset = self.stack_size.unchecked_mul(slot.0);
                        let bottom = self
                            .mapping
                            .as_ptr()
                            .add(offset.byte_count() + self.page_size.byte_count())
                            .cast_mut();

                        match VMContinuationStack::from_raw_parts(
                            bottom,
                            self.page_size.byte_count(),
                            usable_size.byte_count(),
                        ) {
                            Ok(stack) => Ok(Some((stack, slot))),
                            Err(error) => {
                                state.free.push(slot)?;
                                Err(error)
                            }
                        }
                    }
                }

                pub(super) fn deallocate_many(
                    &self,
                    slots: impl IntoIterator<Item = StackSlot>,
                ) {
                    let mut state = self.state.lock();
                    for slot in slots {
                        state
                            .free
                            .push(slot)
                            .expect("arena free list was preallocated to its maximum size");
                    }
                }
            }
        }
    }
    _ => {
        mod imp {
            use super::*;

            #[derive(Clone, Copy, Debug)]
            pub(super) struct StackSlot;

            #[derive(Debug)]
            struct ArenaState {
                live: usize,
            }

            /// Engine-owned concurrency limit for platforms without guarded
            /// continuation-stack slots.
            #[derive(Debug)]
            pub struct ContinuationStackArena {
                stack_size: usize,
                max_stacks: usize,
                state: Mutex<ArenaState>,
            }

            impl ContinuationStackArena {
                pub fn new(config: &PoolingAllocationConfig, stack_size: usize) -> Result<Self> {
                    let max_stacks = usize::try_from(config.limits.total_stacks).unwrap();
                    Ok(Self {
                        stack_size,
                        max_stacks,
                        state: Mutex::new(ArenaState { live: 0 }),
                    })
                }

                pub(super) fn allocate(
                    &self,
                ) -> Result<Option<(VMContinuationStack, StackSlot)>> {
                    let mut state = self.state.lock();
                    if state.live == self.max_stacks {
                        return Ok(None);
                    }
                    let stack = VMContinuationStack::new(self.stack_size)?;
                    state.live += 1;
                    Ok(Some((stack, StackSlot)))
                }

                pub(super) fn deallocate_many(
                    &self,
                    slots: impl IntoIterator<Item = StackSlot>,
                ) {
                    let mut state = self.state.lock();
                    state.live -= slots.into_iter().count();
                }
            }
        }
    }
}

pub use imp::ContinuationStackArena;

/// Store-local ownership and recycling metadata for continuations.
#[derive(Debug)]
pub struct ContinuationStackPool {
    arena: Arc<ContinuationStackArena>,
    owners: StackOwners,
    arena_known_full: bool,
}

impl ContinuationStackPool {
    pub fn new(arena: Arc<ContinuationStackArena>, max_stacks: usize) -> Result<Self> {
        Ok(Self {
            arena,
            owners: StackOwners::with_capacity(max_stacks)?,
            arena_known_full: false,
        })
    }

    pub fn arena_known_full(&self) -> bool {
        self.arena_known_full
    }

    pub fn recycle(&mut self) -> Option<SendSyncPtr<VMContRef>> {
        self.owners.recycle()
    }

    pub fn allocate(
        &mut self,
        owner: SendSyncPtr<VMContRef>,
    ) -> Result<Option<VMContinuationStack>> {
        let Some((stack, slot)) = self.arena.allocate()? else {
            self.arena_known_full = true;
            return Ok(None);
        };
        self.arena_known_full = false;
        if let Err(error) = self.owners.push(owner, slot) {
            self.arena.deallocate_many([slot]);
            return Err(error);
        }
        Ok(Some(stack))
    }
}

impl Drop for ContinuationStackPool {
    fn drop(&mut self) {
        self.arena.deallocate_many(self.owners.slots());
    }
}

#[cfg(all(test, target_os = "linux", target_arch = "x86_64"))]
mod tests {
    use super::*;
    use crate::config::InstanceLimits;

    #[cfg(all(not(miri), not(asan)))]
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
        let arena = Arc::new(ContinuationStackArena::new(&config, 1)?);
        let mut pool = ContinuationStackPool::new(arena.clone(), 0)?;

        assert!(pool.recycle().is_none());
        assert!(arena.allocate()?.is_none());
        Ok(())
    }

    #[cfg(all(not(miri), not(asan)))]
    #[test]
    fn continuation_stack_pool_enforces_its_limit() -> Result<()> {
        let config = PoolingAllocationConfig {
            limits: InstanceLimits {
                total_stacks: 2,
                ..Default::default()
            },
            ..PoolingAllocationConfig::default()
        };
        let arena = Arc::new(ContinuationStackArena::new(&config, 1)?);
        let mut first_owner = Box::new(VMContRef::empty());
        let mut second_owner = Box::new(VMContRef::empty());
        let mut third_owner = Box::new(VMContRef::empty());
        let mut first_pool = ContinuationStackPool::new(arena.clone(), 2)?;
        let mut second_pool = ContinuationStackPool::new(arena, 2)?;

        let first = first_pool.allocate(owner(&mut first_owner))?.unwrap();
        let second = first_pool.allocate(owner(&mut second_owner))?.unwrap();
        assert!(first.is_from_raw_parts());
        assert!(second.is_from_raw_parts());
        assert_ne!(first.range(), second.range());
        let first_range = first.range();
        first_owner.stack = first;
        second_owner.stack = second;

        assert!(first_pool.recycle().is_none());
        assert!(second_pool.allocate(owner(&mut third_owner))?.is_none());
        assert!(second_pool.arena_known_full());

        first_owner.common_stack_information.state = VMStackState::Returned;
        let recycled = first_pool.recycle().unwrap();
        assert_eq!(recycled.as_ptr(), first_owner.as_mut() as *mut VMContRef);
        assert_eq!(first_owner.stack.range(), first_range);

        first_owner.common_stack_information.state = VMStackState::Running;
        second_owner.common_stack_information.state = VMStackState::Trapped;
        let recycled = first_pool.recycle().unwrap();
        assert_eq!(recycled.as_ptr(), second_owner.as_mut() as *mut VMContRef);

        // Dropping a store-local pool returns its engine-owned stack
        // slots for use by other stores. In production the owning
        // continuations are dropped first by `StoreOpaque`'s field
        // order.
        drop(first_owner);
        drop(second_owner);
        drop(first_pool);
        let third = second_pool.allocate(owner(&mut third_owner))?.unwrap();
        assert!(!second_pool.arena_known_full());
        assert!(third.is_from_raw_parts());
        third_owner.stack = third;
        Ok(())
    }
}
