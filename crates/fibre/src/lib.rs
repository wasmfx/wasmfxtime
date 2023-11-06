use std::cell::Cell;
use std::io;
use std::marker::PhantomData;
use std::ops::Range;

cfg_if::cfg_if! {
    if #[cfg(unix)] {
        pub mod unix;
        use unix as imp;
    } else {
        compile_error!("fibers are not supported on this platform");
    }
}

pub type TagId = u32;

/// See SwitchReason for overall use of this type.
#[repr(u32)]
pub enum SwitchReasonEnum {
    // Used to indicate that the contination has returned normally.
    Return = 0,

    // Indicates that we are suspendinga continuation due to invoking suspend.
    // The payload is the tag to suspend with
    Suspend = 1,

    // Indicates that we are resuming a continuation via resume.
    Resume = 2,
}

impl SwitchReasonEnum {
    pub fn discriminant_val(&self) -> u32 {
        // This is well-defined for an enum with repr(u32).
        unsafe { *(self as *const SwitchReasonEnum as *const u32) }
    }
}

/// Values of this type are passed to `wasmtime_fibre_switch` to indicate why we
/// are switching. A nicer way of representing this type would be the following
/// enum:
///
///   #[repr(C)]
///   pub enum SwitchReason {
///       // Used to indicate that the contination has returned normally.
///       Return = 0,
///
///       // Indicates that we are suspendinga continuation due to invoking suspend.
///       // The payload is the tag to suspend with
///       Suspend(u32) = 1,
///
///       // Indicates that we are resuming a continuation via resume.
///       Resume = 2,
///   }
///
/// However, we want to convert values of type `SwitchReason` to and from u64
/// easily, which is why we need to ensure that it contains no uninitialised
/// memory, to avoid undefined behavior.
///
/// We allow converting values of this type to and from u64.
/// In that representation, bits 0 to 31 (where 0 is the LSB) contain the
/// discriminant (as u32), while bits 32 to 63 contain the `data`.
#[repr(C)]
pub struct SwitchReason {
    discriminant: SwitchReasonEnum,

    // Stores tag value if `discriminant` is `suspend`, 0 otherwise.
    data: u32,
}

impl SwitchReason {
    pub fn return_() -> SwitchReason {
        SwitchReason {
            discriminant: SwitchReasonEnum::Return,
            data: 0,
        }
    }

    pub fn resume() -> SwitchReason {
        SwitchReason {
            discriminant: SwitchReasonEnum::Resume,
            data: 0,
        }
    }

    pub fn suspend(tag: u32) -> SwitchReason {
        SwitchReason {
            discriminant: SwitchReasonEnum::Suspend,
            data: tag,
        }
    }
}

impl Into<u64> for SwitchReason {
    fn into(self) -> u64 {
        // TODO(frank-emrich) This assumes little endian data layout. Should
        // make this more explicit.
        unsafe { std::mem::transmute::<SwitchReason, u64>(self) }
    }
}

impl From<u64> for SwitchReason {
    fn from(val: u64) -> SwitchReason {
        #[cfg(debug_assertions)]
        {
            let discriminant = val as u32;
            debug_assert!(discriminant <= 2);
            if discriminant != SwitchReasonEnum::Suspend.discriminant_val() {
                let data = val >> 32;
                debug_assert_eq!(data, 0);
            }
        }
        // TODO(frank-emrich) This assumes little endian data layout. Should
        // make this more explicit.
        unsafe { std::mem::transmute::<u64, SwitchReason>(val) }
    }
}

/// Represents an execution stack to use for a fiber.
#[derive(Debug)]
pub struct FiberStack(imp::FiberStack);

impl FiberStack {
    /// Creates a new fiber stack of the given size.
    pub fn new(size: usize) -> io::Result<Self> {
        Ok(Self(imp::FiberStack::new(size)?))
    }

    /// Creates a new fiber stack of the given size (using malloc).
    pub fn malloc(size: usize) -> io::Result<Self> {
        Ok(Self(imp::FiberStack::malloc(size)?))
    }

    /// Creates a new fiber stack with the given pointer to the bottom of the
    /// stack plus the byte length of the stack.
    ///
    /// The `bottom` pointer should be addressable for `len` bytes. The page
    /// beneath `bottom` should be unmapped as a guard page.
    ///
    /// # Safety
    ///
    /// This is unsafe because there is no validation of the given pointer.
    ///
    /// The caller must properly allocate the stack space with a guard page and
    /// make the pages accessible for correct behavior.
    pub unsafe fn from_raw_parts(bottom: *mut u8, len: usize) -> io::Result<Self> {
        Ok(Self(imp::FiberStack::from_raw_parts(bottom, len)?))
    }

    /// Gets the top of the stack.
    ///
    /// Returns `None` if the platform does not support getting the top of the
    /// stack.
    pub fn top(&self) -> Option<*mut u8> {
        self.0.top()
    }

    /// Returns the range of where this stack resides in memory if the platform
    /// supports it.
    pub fn range(&self) -> Option<Range<usize>> {
        self.0.range()
    }
}

pub struct Fiber {
    stack: FiberStack,
    inner: imp::Fiber,
    done: Cell<bool>,
    _phantom: PhantomData<()>,
}

pub struct Suspend {
    inner: imp::Suspend,
    _phantom: PhantomData<()>,
}

impl Fiber {
    /// Creates a new fiber which will execute `func` on the given stack.
    ///
    /// This function returns a `Fiber` which, when resumed, will execute `func`
    /// to completion. When desired the `func` can suspend itself via
    /// `Fiber::suspend`.
    pub fn new(stack: FiberStack, func: impl FnOnce((), &Suspend) -> ()) -> io::Result<Self> {
        let inner = imp::Fiber::new(&stack.0, func)?;

        Ok(Self {
            stack,
            inner,
            done: Cell::new(false),
            _phantom: PhantomData,
        })
    }

    /// Resumes execution of this fiber.
    ///
    /// This function will transfer execution to the fiber and resume from where
    /// it last left off.
    ///
    /// Returns `true` if the fiber finished or `false` if the fiber was
    /// suspended in the middle of execution.
    ///
    /// # Panics
    ///
    /// Panics if the current thread is already executing a fiber or if this
    /// fiber has already finished.
    ///
    /// Note that if the fiber itself panics during execution then the panic
    /// will be propagated to this caller.
    pub fn resume(&self) -> SwitchReason {
        assert!(!self.done.replace(true), "cannot resume a finished fiber");
        let reason = self.inner.resume(&self.stack.0);
        match reason {
            SwitchReason {
                discriminant: SwitchReasonEnum::Suspend,
                data: _,
            } => self.done.set(false),
            _ => (),
        };
        reason
    }

    /// Returns whether this fiber has finished executing.
    pub fn done(&self) -> bool {
        self.done.get()
    }

    /// Gets the stack associated with this fiber.
    pub fn stack(&self) -> &FiberStack {
        &self.stack
    }
}

impl Suspend {
    /// Suspend execution of a currently running fiber.
    ///
    /// This function will switch control back to the original caller of
    /// `Fiber::resume`. This function will then return once the `Fiber::resume`
    /// function is called again.
    ///
    /// # Panics
    ///
    /// Panics if the current thread is not executing a fiber from this library.
    pub fn suspend(&self, tag: TagId) {
        let reason = SwitchReason::suspend(tag);
        self.inner.switch(reason);
    }

    fn execute(inner: imp::Suspend, func: impl FnOnce((), &Suspend) -> ()) {
        let suspend = Suspend {
            inner,
            _phantom: PhantomData,
        };
        // Note that the original wasmtime-fiber crate runs `func` wrapped in
        // `panic::catch_unwind`, to stop panics from being propagated onward,
        // instead just reporting parent. We eschew this, doing nothing special
        // about panics.
        (func)((), &suspend);
        let reason = SwitchReason::return_();
        suspend.inner.switch(reason);
    }
}

impl Drop for Fiber {
    fn drop(&mut self) {
        debug_assert!(self.done.get(), "fiber dropped without finishing");
    }
}
