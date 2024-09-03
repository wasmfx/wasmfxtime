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
        pub type Length = u32;
        /// type of capacity
        pub type Capacity = u32;
        /// Type of the entries in the actual buffer
        pub type DataEntries = u128;
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

// Since `StackLimits` objects appear in the `StoreOpaque`,
// they need to be `Send` and `Sync`.
// This is safe for the same reason it is for `VMRuntimeLimits` (see comment
// there): Both types are pod-type with no destructor, and we don't access any
// of their fields from other threads.
unsafe impl Send for StackLimits {}
unsafe impl Sync for StackLimits {}

#[repr(C)]
#[derive(Debug, Clone)]
/// This is a Vector-like type. However, we consider `Payloads` objects NOT to
/// own the underlying data buffer. As a result, it does not implement Drop, all
/// (de)allocation must be done manually.
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
    /// Does not preserve existing elements and can therefore only be called on empty `Payloads`.
    pub fn ensure_capacity(&mut self, required_capacity: u32) {
        assert_eq!(self.length, 0);
        if self.capacity < required_capacity {
            self.deallocate();

            *self = Self::new(required_capacity)
        }
    }

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

    pub fn clear(&mut self) {
        self.length = 0;
    }
}

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
    /// The `VMContRef` has been created, but `resume` has never been
    /// called on it. During this stage, we may add arguments using `cont.bind`.
    Allocated,
    /// `resume` has been invoked at least once on the `VMContRef`,
    /// meaning that the function passed to `cont.new` has started executing.
    /// Note that this state does not indicate whether the execution of this
    /// function is currently suspended or not.
    Invoked,
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
    /// Offsets of fields in `Payloads`
    pub mod payloads {
        use crate::Payloads;
        use memoffset::offset_of;

        /// Offset of `capacity` field
        pub const CAPACITY: usize = offset_of!(Payloads, capacity);
        /// Offset of `data` field
        pub const DATA: usize = offset_of!(Payloads, data);
        /// Offset of `length` field
        pub const LENGTH: usize = offset_of!(Payloads, length);
    }

    /// Offsets of fields in `wasmtime_runtime::continuation::VMContRef`.
    /// We uses tests there to ensure these values are correct.
    pub mod vm_cont_ref {
        use crate::Payloads;

        /// Offset of `limits` field
        pub const LIMITS: usize = 0;
        /// Offset of `parent_chain` field
        pub const PARENT_CHAIN: usize = LIMITS + 4 * core::mem::size_of::<usize>();
        /// Offset of `stack` field
        pub const STACK: usize = PARENT_CHAIN + 2 * core::mem::size_of::<usize>();
        /// Offset of `args` field
        pub const ARGS: usize = STACK + super::FIBER_STACK_SIZE;
        /// Offset of `tag_return_values` field
        pub const TAG_RETURN_VALUES: usize = ARGS + core::mem::size_of::<Payloads>();
        /// Offset of `state` field
        pub const STATE: usize = TAG_RETURN_VALUES + core::mem::size_of::<Payloads>();
        /// Offset of `revision` field
        pub const REVISION: usize = STATE + core::mem::size_of::<usize>();
    }

    pub mod stack_limits {
        use crate::StackLimits;
        use memoffset::offset_of;

        pub const STACK_LIMIT: usize = offset_of!(StackLimits, stack_limit);
        pub const LAST_WASM_EXIT_FP: usize = offset_of!(StackLimits, last_wasm_exit_fp);
        pub const LAST_WASM_EXIT_PC: usize = offset_of!(StackLimits, last_wasm_exit_pc);
        pub const LAST_WASM_ENTRY_SP: usize = offset_of!(StackLimits, last_wasm_entry_sp);
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

/// Universal control effect. This structure encodes return signal,
/// resume signal, suspension signal, and suspension tags into a
/// pointer. This instance is used at runtime. There is a codegen
/// counterpart in `cranelift/src/wasmfx/shared.rs`.
#[repr(transparent)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ControlEffect(TaggedPointer);

impl ControlEffect {
    pub fn suspend(ptr: *const u8) -> Self {
        let tptr = TaggedPointer::untagged(ptr as usize);
        Self(TaggedPointer::low_tag(tptr, 0b01_usize))
    }

    pub fn return_() -> Self {
        Self((0b00_usize).into())
    }

    pub fn resume() -> Self {
        Self((0b11_usize).into())
    }

    fn new(raw: usize) -> Self {
        Self(TaggedPointer::untagged(raw))
    }

    pub fn is_suspend(self) -> bool {
        TaggedPointer::get_low_tag(self.0) == 0b01
    }
}

impl From<u64> for ControlEffect {
    fn from(val: u64) -> ControlEffect {
        ControlEffect::new(val as usize)
    }
}

impl From<ControlEffect> for u64 {
    fn from(val: ControlEffect) -> u64 {
        let raw: usize = val.0.into();
        raw as u64
    }
}

impl From<ControlEffect> for *mut u8 {
    fn from(val: ControlEffect) -> *mut u8 {
        let raw: usize = val.0.into();
        raw as *mut u8
    }
}
