#![no_std]
use core::{
    convert::From,
    default::Default,
    marker::{Send, Sync},
    ptr,
};

extern crate alloc;
use alloc::vec::Vec;
use core::mem::drop;

/// Default size for continuation stacks
pub const DEFAULT_FIBER_SIZE: usize = 2097152; // 2MB = 512 pages of 4k

/// Default size of the red zone at the bottom of a fiber stack. This means that
/// whenever we are executing on a fiber stack and starting (!) execution of a
/// wasm (!) function, the stack pointer must be at least this many bytes away
/// from the bottom of the fiber stack.
pub const DEFAULT_RED_ZONE_SIZE: usize = 32768; // 32K = 8 pages of 4k size

/// Capacity of the `HandlerList` initially created for the main stack and every
/// continuation.
pub const INITIAL_HANDLER_LIST_CAPACITY: usize = 4;

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

/// Runtime configuration options for WasmFX that can be set via the command
/// line.
///
/// Part of wasmtime::config::Config type (which is not in scope in this crate).
#[derive(Debug, Clone)]
pub struct WasmFXConfig {
    pub stack_size: usize,

    /// Space that must be left on stack when starting execution of a
    /// function while running on a continuation stack.
    /// Must be smaller than the value of `stack_size` above.
    pub red_zone_size: usize,
}

/// This type is used to save (and subsequently restore) a subset of the data in
/// `VMRuntimeLimits`. See documentation of `StackChain` for the exact uses.
#[repr(C)]
#[derive(Debug, Default, Clone)]
pub struct StackLimits {
    pub stack_limit: usize,
    pub last_wasm_entry_fp: usize,
}

/// This type represents "common" information that we need to save both for the
/// main stack and each continuation.
#[repr(C)]
#[derive(Debug, Clone)]
pub struct CommonStackInformation {
    pub limits: StackLimits,
    /// For the main stack, this field must only have one of the following values:
    /// - Running
    /// - Parent
    pub state: State,

    /// Only in use when state is `Parent`.
    /// Otherwise, the list may remain allocated, but its `length` must be 0.
    ///
    /// Represents the handlers that this stack installed when resume-ing a
    /// continuation.
    ///
    /// Note that for any resume instruction, we can re-order the handler
    /// clauses without changing behavior such that all the suspend handlers
    /// come first, followed by all the switch handler (while maintaining the
    /// original ordering within the two groups).
    /// Thus, we assume that the given resume instruction has the following
    /// shape:
    ///
    /// (resume $ct
    ///   (on $tag_0 $block_0) ... (on $tag_{n-1} $block_{n-1})
    ///   (on $tag_n switch) ... (on $tag_m switch)
    /// )
    ///
    /// On resume, the handler list is then filled with m + 1 (i.e., one per
    /// handler clause) entries such that the i-th entry, using 0-based
    /// indexing, is the identifier of $tag_i (represented as *mut
    /// VMTagDefinition).
    /// Further, `first_switch_handler_index` (see below) is set to n (i.e., the
    /// 0-based index of the first switch handler).
    pub handlers: HandlerList,

    /// Only used when state is `Parent`. See documentation of `handlers` above.
    pub first_switch_handler_index: u32,
}

impl CommonStackInformation {
    pub fn running_default() -> Self {
        Self {
            limits: StackLimits::default(),
            state: State::Running,
            handlers: HandlerList::new(INITIAL_HANDLER_LIST_CAPACITY as u32),
            first_switch_handler_index: 0,
        }
    }
}

impl StackLimits {
    pub fn with_stack_limit(stack_limit: usize) -> Self {
        Self {
            stack_limit,
            ..Default::default()
        }
    }
}

// Since `StackLimits` objects appear in the `StoreOpaque`,
// they need to be `Send` and `Sync`.
// This is safe for the same reason it is for `VMRuntimeLimits` (see comment
// there): Both types are pod-type with no destructor, and we don't access any
// of their fields from other threads.
unsafe impl Send for StackLimits {}
unsafe impl Sync for StackLimits {}

// Same for HandlerList: They appear in the `StoreOpaque`.
unsafe impl Send for HandlerList {}
unsafe impl Sync for HandlerList {}

#[repr(C)]
#[derive(Debug, Clone)]
/// A growable container type. Unlike Rust's `Vec`, we consider `Vector`
/// objects NOT to own the underlying data buffer. As a result, it does not
/// implement `Drop`, all (de)allocation must be done manually.
pub struct Vector<T> {
    /// Number of currently occupied slots.
    pub length: u32,
    /// Number of slots in the data buffer. Note that this is *not* the size of
    /// the buffer in bytes!
    pub capacity: u32,
    /// This is null if and only if capacity (and thus also `length`) are 0.
    pub data: *mut T,
}

impl<T> Vector<T> {
    #[inline]
    pub fn new(capacity: u32) -> Self {
        let data = if capacity == 0 {
            ptr::null_mut()
        } else {
            let mut args = Vec::with_capacity(capacity as usize);
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

    /// Ensures that we can hold at least the required number of elements.
    /// Does not preserve existing elements and can therefore only be called on empty `Vector`.
    #[inline]
    pub fn ensure_capacity(&mut self, required_capacity: u32) {
        assert_eq!(self.length, 0);
        if self.capacity < required_capacity {
            self.deallocate();

            *self = Self::new(required_capacity)
        }
    }
    #[inline]
    pub fn deallocate(&mut self) {
        if self.data.is_null() {
            debug_assert_eq!(self.length, 0);
            debug_assert_eq!(self.capacity, 0);
        } else {
            drop(unsafe {
                Vec::from_raw_parts(self.data, self.length as usize, self.capacity as usize)
            });

            // Just for safety:
            self.data = core::ptr::null_mut();
            self.capacity = 0;
            self.length = 0;
        }
    }

    #[inline]
    pub fn clear(&mut self) {
        self.length = 0;
    }
}

/// Type of vectors used for handling payloads passed between continuations.
///
/// The actual type argument should be wasmtime::runtime::vm::vmcontext::ValRaw,
/// but we don't have access to that here.
pub type Payloads = Vector<u128>;

/// List of handlers, represented by the handled tag.
/// Thus, the stored data is actually `*mut VMTagDefinition`.
pub type HandlerList = Vector<*mut u8>;

/// Discriminant of variant `Absent` in
/// `wasmtime_runtime::continuation::StackChain`.
pub const STACK_CHAIN_ABSENT_DISCRIMINANT: usize = 0;
/// Discriminant of variant `MainStack` in
/// `wasmtime_runtime::continuation::StackChain`.
pub const STACK_CHAIN_MAIN_STACK_DISCRIMINANT: usize = 1;
/// Discriminant of variant `Continiation` in
/// `wasmtime_runtime::continuation::StackChain`.
pub const STACK_CHAIN_CONTINUATION_DISCRIMINANT: usize = 2;

/// Encodes the life cycle of a `VMContRef`.
#[derive(Debug, Clone, Copy, PartialEq)]
#[repr(i32)]
pub enum State {
    /// The `VMContRef` has been created, but neither `resume` or `switch` has ever been
    /// called on it. During this stage, we may add arguments using `cont.bind`.
    Fresh,
    /// The continuation is running, meaning that it is the one currently
    /// executing code.
    Running,
    /// The continuation is suspended because it executed a resume instruction
    /// that has not finished yet. In other words, it became the parent of
    /// another continuation (which may itself be `Running`, a `Parent`, or
    /// `Suspended`).
    Parent,
    /// The continuation was suspended by a `suspend` or `switch` instruction.
    Suspended,
    /// The function originally passed to `cont.new` has returned normally.
    /// Note that there is no guarantee that a VMContRef will ever
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

/// Defines offsets of the fields in the continuation-related types
pub mod offsets {
    /// Offsets of fields in `Vector`.
    /// Note that these are independent from the type parameter `T`.
    pub mod vector {
        use crate::Vector;
        use memoffset::offset_of;

        /// Offset of `capacity` field
        pub const CAPACITY: usize = offset_of!(Vector<()>, capacity);
        /// Offset of `data` field
        pub const DATA: usize = offset_of!(Vector<()>, data);
        /// Offset of `length` field
        pub const LENGTH: usize = offset_of!(Vector<()>, length);
    }

    /// Offsets of fields in `wasmtime_runtime::continuation::VMContRef`.
    /// We uses tests there to ensure these values are correct.
    pub mod vm_cont_ref {
        use crate::{CommonStackInformation, Payloads};

        /// Offset of `common_stack_information` field
        pub const COMMON_STACK_INFORMATION: usize = 0;
        /// Offset of `parent_chain` field
        pub const PARENT_CHAIN: usize =
            COMMON_STACK_INFORMATION + core::mem::size_of::<CommonStackInformation>();
        /// Offset of `last_ancestor` field
        pub const LAST_ANCESTOR: usize = PARENT_CHAIN + 2 * core::mem::size_of::<usize>();
        /// Offset of `stack` field
        pub const STACK: usize = LAST_ANCESTOR + core::mem::size_of::<usize>();
        /// Offset of `args` field
        pub const ARGS: usize = STACK + super::FIBER_STACK_SIZE;
        /// Offset of `values` field
        pub const VALUES: usize = ARGS + core::mem::size_of::<Payloads>();
        /// Offset of `revision` field
        pub const REVISION: usize = VALUES + core::mem::size_of::<Payloads>();
    }

    pub mod stack_limits {
        use crate::StackLimits;
        use memoffset::offset_of;

        pub const STACK_LIMIT: usize = offset_of!(StackLimits, stack_limit);
        pub const LAST_WASM_ENTRY_FP: usize = offset_of!(StackLimits, last_wasm_entry_fp);
    }

    pub mod common_stack_information {
        use crate::CommonStackInformation;
        use memoffset::offset_of;

        pub const LIMITS: usize = offset_of!(CommonStackInformation, limits);
        pub const STATE: usize = offset_of!(CommonStackInformation, state);
        pub const HANDLERS: usize = offset_of!(CommonStackInformation, handlers);
        pub const FIRST_SWITCH_HANDLER_INDEX: usize =
            offset_of!(CommonStackInformation, first_switch_handler_index);
    }

    /// Size of wasmtime_runtime::continuation::FiberStack.
    /// We test there that this value is correct.
    pub const FIBER_STACK_SIZE: usize = 3 * core::mem::size_of::<usize>();

    /// Size of type `wasmtime_runtime::continuation::StackChain`.
    /// We test there that this value is correct.
    pub const STACK_CHAIN_SIZE: usize = 2 * core::mem::size_of::<usize>();
}

#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct TaggedPointer(usize);

#[allow(dead_code)]
impl TaggedPointer {
    const LOW_TAG_BITS: usize = 2;
    const LOW_TAG_MASK: usize = (1 << Self::LOW_TAG_BITS) - 1;

    pub fn untagged(val: usize) -> Self {
        Self(val)
    }

    pub fn low_tag(self, tag: usize) -> Self {
        assert!(tag <= Self::LOW_TAG_MASK);
        Self(self.0 | tag)
    }

    pub fn get_low_tag(self) -> usize {
        self.0 & Self::LOW_TAG_MASK
    }

    pub fn low_untag(self) -> Self {
        Self(self.0 & !Self::LOW_TAG_MASK)
    }
}

impl From<TaggedPointer> for usize {
    fn from(val: TaggedPointer) -> usize {
        val.0
    }
}

impl From<usize> for TaggedPointer {
    fn from(val: usize) -> TaggedPointer {
        TaggedPointer::untagged(val)
    }
}

/// Discriminant of variant `Return` in
/// `ControlEffect`.
pub const CONTROL_EFFECT_RETURN_DISCRIMINANT: u32 = 0;
/// Discriminant of variant `Resume` in
/// `ControlEffect`.
pub const CONTROL_EFFECT_RESUME_DISCRIMINANT: u32 = 1;
/// Discriminant of variant `Suspend` in
/// `ControlEffect`.
pub const CONTROL_EFFECT_SUSPEND_DISCRIMINANT: u32 = 2;
/// Discriminant of variant `Switch` in
/// `ControlEffect`.
pub const CONTROL_EFFECT_SWITCH_DISCRIMINANT: u32 = 3;

/// Universal control effect. This structure encodes return signal,
/// resume signal, suspension signal, and the handler to suspend to in a single variant type.
/// This instance is used at runtime. There is a codegen
/// counterpart in `cranelift/src/wasmfx/shared.rs`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ControlEffect {
    Return = CONTROL_EFFECT_RETURN_DISCRIMINANT,
    Resume = CONTROL_EFFECT_RESUME_DISCRIMINANT,
    Suspend { handler_index: u32 } = CONTROL_EFFECT_SUSPEND_DISCRIMINANT,
    Switch = CONTROL_EFFECT_SWITCH_DISCRIMINANT,
}

// TODO(frank-emrich) This conversion assumes little-endian data layout.
// We convert to and from u64 as follows: The 4 LSBs of the u64 are the
// discriminant, the 4 MSBs are the handler_index (if `Suspend`)
impl From<u64> for ControlEffect {
    fn from(val: u64) -> ControlEffect {
        unsafe { core::mem::transmute::<u64, ControlEffect>(val) }
    }
}

// TODO(frank-emrich) This conversion assumes little-endian data layout.
// We convert to and from u64 as follows: The 4 LSBs of the u64 are the
// discriminant, the 4 MSBs are the handler_index (if `Suspend`)
impl From<ControlEffect> for u64 {
    fn from(val: ControlEffect) -> u64 {
        unsafe { core::mem::transmute::<ControlEffect, u64>(val) }
    }
}
