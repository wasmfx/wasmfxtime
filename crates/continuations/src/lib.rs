use std::ptr;
use wasmtime_fibre::Fiber;

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
}

pub type ContinuationFiber = Fiber<'static, (), u32, ()>;

/// This type is used to save (and subsequently restore) a subset of the data in
/// `VMRuntimeLimits`. See documentation of `StackChain` for the exact uses.
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct StackLimits {
    pub stack_limit: usize,
    pub last_wasm_exit_fp: usize,
    pub last_wasm_exit_pc: usize,
    pub last_wasm_entry_sp: usize,
}

impl StackLimits {
    pub fn uninitialized() -> Self {
        Self {
            stack_limit: 0,
            last_wasm_exit_fp: 0,
            last_wasm_exit_pc: 0,
            last_wasm_entry_sp: 0,
        }
    }

    pub fn with_stack_limit(stack_limit: usize) -> Self {
        Self {
            stack_limit,
            last_wasm_exit_fp: 0,
            last_wasm_exit_pc: 0,
            last_wasm_entry_sp: 0,
        }
    }

    pub fn is_uninitialized(&self) -> bool {
        self == &Self::uninitialized()
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
// FIXME(frank-emrich) Note that the data within the StackLimits objects is
// currently not used or updated in any way.
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
}

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
