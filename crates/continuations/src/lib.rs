use std::{cell::UnsafeCell, ptr};
use wasmtime_fibre::Fiber;

pub use wasmtime_fibre::{SwitchDirection, SwitchDirectionEnum, TagId};

/// Default size for continuation stacks
pub const DEFAULT_FIBER_SIZE: usize = 2097152; // 2MB = 512 pages of 4k

/// TODO
#[allow(dead_code)]
pub const ENABLE_DEBUG_PRINTING: bool = false;

#[macro_export]
macro_rules! debug_println {
    ($( $args:expr ),+ ) => {
        #[cfg(debug_assertions)]
        if ENABLE_DEBUG_PRINTING {
            println!($($args),*);
        }
    }
}

/// Makes the types available that we use for various fields.
pub mod types {
    /// Types used by `Payloads` struct
    pub mod payloads {
        /// type of length
        pub type Length = usize;
        /// type of capacity
        pub type Capacity = usize;
        /// Type of the entries in the actual buffer
        pub type DataEntries = u128;
    }

    /// Types used by `wasmtime_fibre::SwitchDirection` struct
    pub mod switch_reason {
        /// Type of `discriminant` field
        pub type Discriminant = u32;

        /// Type of `data` field
        pub type Data = u32;
    }
}

/// Runtime configuration options for WasmFX that can be set via the command
/// line.
///
/// Part of wasmtime::config::Config type (which is not in scope in this crate).
#[derive(Clone)]
pub struct WasmFXConfig {
    pub stack_size: usize,
}

pub type ContinuationFiber = Fiber;

/// This type is used to save (and subsequently restore) a subset of the data in
/// `VMRuntimeLimits`. See documentation of `StackChain` for the exact uses.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct StackLimits {
    pub stack_limit: usize,
    pub last_wasm_exit_fp: usize,
    pub last_wasm_exit_pc: usize,
    pub last_wasm_entry_sp: usize,
}

impl StackLimits {
    pub fn with_stack_limit(stack_limit: usize) -> Self {
        Self {
            stack_limit,
            ..Default::default()
        }
    }
}

const STACK_CHAIN_ABSENT_DISCRIMINANT: usize = 0;
const STACK_CHAIN_MAIN_STACK_DISCRIMINANT: usize = 1;
const STACK_CHAIN_CONTINUATION_DISCRIMINANT: usize = 2;

/// This type represents a linked lists of stacks, additionally associating a
/// `StackLimits` object with each element of the list. Here, a "stack" is
/// either a continuation or the main stack. Note that the linked list character
/// arises from the fact that `StackChain::Continuation` variants have a pointer
/// to have `ContinuationObject`, which in turn has a `parent_chain` value of
/// type `StackChain`.
///
/// There are generally two uses of such chains:
///
/// 1. The `typed_continuations_chain` field in the VMContext contains such a
/// chain of stacks, where the head of the list denotes the stack that is
/// currently executing (either a continuation or the main stack), as well as
/// the parent stacks, in case of a continuation currently running. Note that in
/// this case, the linked list must contains 0 or more `Continuation` elements,
/// followed by a final `MainStack` element. In particular, this list always
/// ends with `MainStack` and never contains an `Absent` variant.
///
/// 2. When a continuation is suspended, its chain of parents eventually ends
/// with an `Absent` variant in its `parent_chain` field. Note that a suspended
/// continuation never appears in the stack chain in the VMContext!
///
///
/// As mentioned before, each stack in a `StackChain` has a corresponding
/// `StackLimits` object. For continuations, this is stored in the `limits`
/// fields of the corresponding `ContinuationObject`. For the main stack, the
/// `MainStack` variant contains a pointer to the
/// `typed_continuations_main_stack_limits` field of the VMContext.
///
/// The following invariants hold for these `StackLimits` objects, and the data
/// in `VMRuntimeLimits`.
///
/// Currently executing stack:
/// For the currently executing stack (i.e., the stack that is at the head of
/// the VMContext's `typed_continuations_chain` list), the associated
/// `StackLimits` object contains stale/undefined data. Instead, the live data
/// describing the limits for the currently executing stack is always maintained
/// in `VMRuntimeLimits`. Note that as a general rule independently from any
/// execution of continuations, the `last_wasm_exit*` fields in the
/// `VMRuntimeLimits` contain undefined values while executing wasm.
///
/// Parents of currently executing stack:
/// For stacks that appear in the tail of the VMContext's
/// `typed_continuations_chain` list (i.e., stacks that are not currently
/// executing themselves, but are a parent of the currently executing stack), we
/// have the following: All the fields in the stack's StackLimits are valid,
/// describing the stack's stack limit, and pointers where executing for that
/// stack entered and exited WASM.
///
/// Suspended continuations:
/// For suspended continuations (including their parents), we have the
/// following. Note that the main stack can never be in this state. The
/// `stack_limit` and `last_enter_wasm_sp` fields of the corresponding
/// `StackLimits` object contain valid data, while the `last_exit_wasm_*` fields
/// contain arbitrary values.
/// There is only one exception to this: Note that a continuation that has been
/// created with cont.new, but never been resumed so far, is considered
/// "suspended". However, its `last_enter_wasm_sp` field contains undefined
/// data. This is justified, because when resume-ing a continuation for the
/// first time, a native-to-wasm trampoline is called, which sets up the
/// `last_wasm_entry_sp` in the `VMRuntimeLimits` with the correct value, thus
/// restoring the necessary invariant.
#[derive(Debug, Clone, PartialEq)]
#[repr(usize, C)]
pub enum StackChain {
    /// If stored in the VMContext, used to indicate that the MainStack entry
    /// has not been set, yet. If stored in a ContinuationObject's parent_chain
    /// field, means that there is currently no parent.
    Absent = STACK_CHAIN_ABSENT_DISCRIMINANT,
    MainStack(*mut StackLimits) = STACK_CHAIN_MAIN_STACK_DISCRIMINANT,
    Continuation(*mut ContinuationObject) = STACK_CHAIN_CONTINUATION_DISCRIMINANT,
}

impl StackChain {
    pub const ABSENT_DISCRIMINANT: usize = STACK_CHAIN_ABSENT_DISCRIMINANT;
    pub const MAIN_STACK_DISCRIMINANT: usize = STACK_CHAIN_MAIN_STACK_DISCRIMINANT;
    pub const CONTINUATION_DISCRIMINANT: usize = STACK_CHAIN_CONTINUATION_DISCRIMINANT;

    pub fn is_main_stack(&self) -> bool {
        matches!(self, StackChain::MainStack(_))
    }

    // We don't implement IntoIterator because our iterator is unsafe, so at
    // least this gives us some way of indicating this, even though the actual
    // unsafety lies in the `next` function.
    pub unsafe fn into_iter(self) -> ContinuationChainIterator {
        ContinuationChainIterator(self)
    }
}

pub struct ContinuationChainIterator(StackChain);

impl Iterator for ContinuationChainIterator {
    type Item = (Option<*mut ContinuationObject>, *mut StackLimits);

    fn next(&mut self) -> Option<Self::Item> {
        match self.0 {
            StackChain::Absent => None,
            StackChain::MainStack(ms) => {
                let next = (None, ms);
                self.0 = StackChain::Absent;
                Some(next)
            }
            StackChain::Continuation(ptr) => {
                let continuation = unsafe { ptr.as_mut().unwrap() };
                let next = (Some(ptr), (&mut continuation.limits) as *mut StackLimits);
                self.0 = continuation.parent_chain.clone();
                Some(next)
            }
        }
    }
}

#[repr(transparent)]
pub struct StackChainCell(pub UnsafeCell<StackChain>);

impl StackChainCell {
    pub fn absent() -> Self {
        StackChainCell(UnsafeCell::new(StackChain::Absent))
    }
}

// Since `StackChainCell` and `StackLimits` objects appear in the `StoreOpaque`,
// they need to be `Send` and `Sync`.
// This is safe for the same reason it is for `VMRuntimeLimits` (see comment
// there): Both types are pod-type with no destructor, and we don't access any
// of their fields from other threads.
unsafe impl Send for StackLimits {}
unsafe impl Sync for StackLimits {}
unsafe impl Send for StackChainCell {}
unsafe impl Sync for StackChainCell {}

pub struct Payloads {
    /// Number of currently occupied slots.
    pub length: types::payloads::Length,
    /// Number of slots in the data buffer. Note that this is *not* the size of
    /// the buffer in bytes!
    pub capacity: types::payloads::Capacity,
    /// This is null if and only if capacity (and thus also `length`) are 0.
    pub data: *mut u128,
}

impl Payloads {
    pub fn new(capacity: usize) -> Self {
        let data = if capacity == 0 {
            ptr::null_mut()
        } else {
            let mut args = Vec::with_capacity(capacity);
            let args_ptr = args.as_mut_ptr();
            args.leak();
            args_ptr
        };
        Self {
            length: 0,
            capacity,
            data,
        }
    }
}

/// Encodes the life cycle of a `ContinuationObject`.
#[derive(PartialEq)]
#[repr(i32)]
pub enum State {
    /// The `ContinuationObject` has been created, but `resume` has never been
    /// called on it. During this stage, we may add arguments using `cont.bind`.
    Allocated,
    /// `resume` has been invoked at least once on the `ContinuationObject`,
    /// meaning that the function passed to `cont.new` has started executing.
    /// Note that this state does not indicate whether the execution of this
    /// function is currently suspended or not.
    Invoked,
    /// The function originally passed to `cont.new` has returned normally.
    /// Note that there is no guarantee that a ContinuationObject will ever
    /// reach this status, as it may stay suspended until being dropped.
    Returned,
}

impl State {
    pub fn discriminant(&self) -> i32 {
        // This is well-defined for an enum with repr(i32).
        unsafe { *(self as *const Self as *const i32) }
    }
}

impl From<State> for i32 {
    fn from(st: State) -> Self {
        st.discriminant()
    }
}

/// TODO
#[repr(C)]
pub struct ContinuationObject {
    pub limits: StackLimits,

    pub parent_chain: StackChain,

    pub fiber: *mut ContinuationFiber,

    /// Used to store
    /// 1. The arguments to the function passed to cont.new
    /// 2. The return values of that function
    /// Note that this is *not* used for tag payloads.
    pub args: Payloads,

    // Once a continuation is suspended, this buffer is used to hold payloads
    // provided by cont.bind and resume and received at the suspend site.
    // In particular, this may only be Some when `state` is `Invoked`.
    pub tag_return_values: Payloads,

    pub state: State,
}

/// M:1 Many-to-one mapping. A single ContinuationObject may be
/// referenced by multiple ContinuationReference, though, only one
/// ContinuationReference may hold a non-null reference to the object
/// at a given time.
#[repr(C)]
pub struct ContinuationReference(pub Option<*mut ContinuationObject>);

/// Defines offsets of the fields in the types defined earlier
pub mod offsets {
    /// Offsets of fields in `Payloads`
    pub mod payloads {
        use crate::Payloads;
        use memoffset::offset_of;

        /// Offset of `capacity` field
        pub const CAPACITY: i32 = offset_of!(Payloads, capacity) as i32;
        /// Offset of `data` field
        pub const DATA: i32 = offset_of!(Payloads, data) as i32;
        /// Offset of `length` field
        pub const LENGTH: i32 = offset_of!(Payloads, length) as i32;
    }

    /// Offsets of fields in `ContinuationObject`
    pub mod continuation_object {
        use crate::ContinuationObject;
        use memoffset::offset_of;

        /// Offset of `limits` field
        pub const LIMITS: i32 = offset_of!(ContinuationObject, limits) as i32;
        /// Offset of `args` field
        pub const ARGS: i32 = offset_of!(ContinuationObject, args) as i32;
        /// Offset of `parent_chain` field
        pub const PARENT_CHAIN: i32 = offset_of!(ContinuationObject, parent_chain) as i32;
        /// Offset of `state` field
        pub const STATE: i32 = offset_of!(ContinuationObject, state) as i32;
        /// Offset of `tag_return_values` field
        pub const TAG_RETURN_VALUES: i32 = offset_of!(ContinuationObject, tag_return_values) as i32;
    }

    pub mod stack_limits {
        use crate::StackLimits;
        use memoffset::offset_of;

        pub const STACK_LIMIT: i32 = offset_of!(StackLimits, stack_limit) as i32;
        pub const LAST_WASM_EXIT_FP: i32 = offset_of!(StackLimits, last_wasm_exit_fp) as i32;
        pub const LAST_WASM_EXIT_PC: i32 = offset_of!(StackLimits, last_wasm_exit_pc) as i32;
        pub const LAST_WASM_ENTRY_SP: i32 = offset_of!(StackLimits, last_wasm_entry_sp) as i32;
    }
}
