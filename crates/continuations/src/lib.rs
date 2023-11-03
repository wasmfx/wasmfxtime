use num_enum::{IntoPrimitive, TryFromPrimitive};
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
    pub fn new(capacity: usize) -> Payloads {
        let data = if capacity == 0 {
            ptr::null_mut()
        } else {
            let mut args = Vec::with_capacity(capacity);
            let args_ptr = args.as_mut_ptr();
            args.leak();
            args_ptr
        };
        return Payloads {
            length: 0,
            capacity,
            data,
        };
    }
}

/// Encodes the life cycle of a `ContinuationObject`.
#[derive(PartialEq, IntoPrimitive, TryFromPrimitive)]
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

/// TODO
#[repr(C)]
pub struct ContinuationObject {
    pub parent: *mut ContinuationObject,

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

        /// Offset of `args` field
        pub const ARGS: i32 = offset_of!(ContinuationObject, args) as i32;
        /// Offset of `parent` field
        pub const PARENT: i32 = offset_of!(ContinuationObject, parent) as i32;
        /// Offset of `state` field
        pub const STATE: i32 = offset_of!(ContinuationObject, state) as i32;
        /// Offset of `tag_return_values` field
        pub const TAG_RETURN_VALUES: i32 = offset_of!(ContinuationObject, tag_return_values) as i32;
    }
}
