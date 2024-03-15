use std::ptr;

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
        pub type Length = u32;
        /// type of capacity
        pub type Capacity = u32;
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

// Runtime representation of tags
pub type TagId = u32;

/// See SwitchDirection below for overall use of this type.
#[repr(u32)]
pub enum SwitchDirectionEnum {
    // Used to indicate that the contination has returned normally.
    Return = 0,

    // Indicates that we are suspendinga continuation due to invoking suspend.
    // The payload is the tag to suspend with
    Suspend = 1,

    // Indicates that we are resuming a continuation via resume.
    Resume = 2,
}

impl SwitchDirectionEnum {
    pub fn discriminant_val(&self) -> u32 {
        // This is well-defined for an enum with repr(u32).
        unsafe { *(self as *const SwitchDirectionEnum as *const u32) }
    }
}

/// Values of this type are passed to `wasmtime_fibre_switch` to indicate why we
/// are switching. A nicer way of representing this type would be the following
/// enum:
///
///```
///  #[repr(C, u32)]
///  pub enum SwitchDirection {
///      // Used to indicate that the contination has returned normally.
///      Return = 0,
///
///      // Indicates that we are suspendinga continuation due to invoking suspend.
///      // The payload is the tag to suspend with
///      Suspend(u32) = 1,
///
///      // Indicates that we are resuming a continuation via resume.
///      Resume = 2,
///  }
///```
///
/// However, we want to convert values of type `SwitchDirection` to and from u64
/// easily, which is why we need to ensure that it contains no uninitialised
/// memory, to avoid undefined behavior.
///
/// We allow converting values of this type to and from u64.
/// In that representation, bits 0 to 31 (where 0 is the LSB) contain the
/// discriminant (as u32), while bits 32 to 63 contain the `data`.
#[repr(C)]
pub struct SwitchDirection {
    pub discriminant: SwitchDirectionEnum,

    // Stores tag value if `discriminant` is `suspend`, 0 otherwise.
    pub data: u32,
}

impl SwitchDirection {
    pub fn return_() -> SwitchDirection {
        SwitchDirection {
            discriminant: SwitchDirectionEnum::Return,
            data: 0,
        }
    }

    pub fn resume() -> SwitchDirection {
        SwitchDirection {
            discriminant: SwitchDirectionEnum::Resume,
            data: 0,
        }
    }

    pub fn suspend(tag: u32) -> SwitchDirection {
        SwitchDirection {
            discriminant: SwitchDirectionEnum::Suspend,
            data: tag,
        }
    }
}

impl From<SwitchDirection> for u64 {
    fn from(val: SwitchDirection) -> u64 {
        // TODO(frank-emrich) This assumes little endian data layout. Should
        // make this more explicit.
        unsafe { std::mem::transmute::<SwitchDirection, u64>(val) }
    }
}

impl From<u64> for SwitchDirection {
    fn from(val: u64) -> SwitchDirection {
        #[cfg(debug_assertions)]
        {
            let discriminant = val as u32;
            debug_assert!(discriminant <= 2);
            if discriminant != SwitchDirectionEnum::Suspend.discriminant_val() {
                let data = val >> 32;
                debug_assert_eq!(data, 0);
            }
        }
        // TODO(frank-emrich) This assumes little endian data layout. Should
        // make this more explicit.
        unsafe { std::mem::transmute::<u64, SwitchDirection>(val) }
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

    /// Offsets of fields in `wasmtime_runtime::continuation::ContinuationObject`.
    /// We uses tests there to ensure these values are correct.
    pub mod continuation_object {
        use crate::Payloads;

        /// Offset of `limits` field
        pub const LIMITS: usize = 0;
        /// Offset of `parent_chain` field
        pub const PARENT_CHAIN: usize = LIMITS + 4 * std::mem::size_of::<usize>();
        /// Offset of `fiber` field
        pub const FIBER: usize = PARENT_CHAIN + 2 * std::mem::size_of::<usize>();
        /// Offset of `args` field
        pub const ARGS: usize = FIBER + std::mem::size_of::<usize>();
        /// Offset of `tag_return_values` field
        pub const TAG_RETURN_VALUES: usize = ARGS + std::mem::size_of::<Payloads>();
        /// Offset of `state` field
        pub const STATE: usize = TAG_RETURN_VALUES + std::mem::size_of::<Payloads>();
    }

    pub mod stack_limits {
        use crate::StackLimits;
        use memoffset::offset_of;

        pub const STACK_LIMIT: usize = offset_of!(StackLimits, stack_limit);
        pub const LAST_WASM_EXIT_FP: usize = offset_of!(StackLimits, last_wasm_exit_fp);
        pub const LAST_WASM_EXIT_PC: usize = offset_of!(StackLimits, last_wasm_exit_pc);
        pub const LAST_WASM_ENTRY_SP: usize = offset_of!(StackLimits, last_wasm_entry_sp);
    }

    /// Size of type `wasmtime_runtime::continuation::StackChain`.
    /// We test there that this value is correct.
    pub const STACK_CHAIN_SIZE: usize = 2 * std::mem::size_of::<usize>();
}
