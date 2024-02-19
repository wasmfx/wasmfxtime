//! WebAssembly trap handling, which is built on top of the lower-level
//! signalhandling mechanisms.

mod backtrace;
mod coredump;

use crate::sys::traphandlers;
use crate::{Instance, VMContext, VMRuntimeLimits};
use anyhow::Error;
use std::any::Any;
use std::cell::{Cell, UnsafeCell};
use std::mem::MaybeUninit;
use std::ptr;
use std::sync::Once;

pub use self::backtrace::{Backtrace, Frame};
pub use self::coredump::CoreDumpStack;
pub use self::tls::{tls_eager_initialize, AsyncWasmCallState, PreviousAsyncWasmCallState};

pub use traphandlers::SignalHandler;

/// Globally-set callback to determine whether a program counter is actually a
/// wasm trap.
///
/// This is initialized during `init_traps` below. The definition lives within
/// `wasmtime` currently.
pub(crate) static mut IS_WASM_PC: fn(usize) -> bool = |_| false;

/// This function is required to be called before any WebAssembly is entered.
/// This will configure global state such as signal handlers to prepare the
/// process to receive wasm traps.
///
/// This function must not only be called globally once before entering
/// WebAssembly but it must also be called once-per-thread that enters
/// WebAssembly. Currently in wasmtime's integration this function is called on
/// creation of a `Engine`.
///
/// The `is_wasm_pc` argument is used when a trap happens to determine if a
/// program counter is the pc of an actual wasm trap or not. This is then used
/// to disambiguate faults that happen due to wasm and faults that happen due to
/// bugs in Rust or elsewhere.
pub fn init_traps(is_wasm_pc: fn(usize) -> bool, macos_use_mach_ports: bool) {
    static INIT: Once = Once::new();

    INIT.call_once(|| unsafe {
        IS_WASM_PC = is_wasm_pc;
        traphandlers::platform_init(macos_use_mach_ports);
    });

    #[cfg(target_os = "macos")]
    assert_eq!(
        traphandlers::using_mach_ports(),
        macos_use_mach_ports,
        "cannot configure two different methods of signal handling in the same process"
    );
}

fn lazy_per_thread_init() {
    traphandlers::lazy_per_thread_init();
}

/// Raises a trap immediately.
///
/// This function performs as-if a wasm trap was just executed. This trap
/// payload is then returned from `catch_traps` below.
///
/// # Safety
///
/// Only safe to call when wasm code is on the stack, aka `catch_traps` must
/// have been previously called. Additionally no Rust destructors can be on the
/// stack. They will be skipped and not executed.
pub unsafe fn raise_trap(reason: TrapReason) -> ! {
    tls::with(|info| info.unwrap().unwind_with(UnwindReason::Trap(reason)))
}

/// Raises a user-defined trap immediately.
///
/// This function performs as-if a wasm trap was just executed, only the trap
/// has a dynamic payload associated with it which is user-provided. This trap
/// payload is then returned from `catch_traps` below.
///
/// # Safety
///
/// Only safe to call when wasm code is on the stack, aka `catch_traps` must
/// have been previously called. Additionally no Rust destructors can be on the
/// stack. They will be skipped and not executed.
pub unsafe fn raise_user_trap(error: Error, needs_backtrace: bool) -> ! {
    raise_trap(TrapReason::User {
        error,
        needs_backtrace,
    })
}

/// Raises a trap from inside library code immediately.
///
/// This function performs as-if a wasm trap was just executed. This trap
/// payload is then returned from `catch_traps` below.
///
/// # Safety
///
/// Only safe to call when wasm code is on the stack, aka `catch_traps` must
/// have been previously called. Additionally no Rust destructors can be on the
/// stack. They will be skipped and not executed.
pub unsafe fn raise_lib_trap(trap: wasmtime_environ::Trap) -> ! {
    raise_trap(TrapReason::Wasm(trap))
}

/// Carries a Rust panic across wasm code and resumes the panic on the other
/// side.
///
/// # Safety
///
/// Only safe to call when wasm code is on the stack, aka `catch_traps` must
/// have been previously called. Additionally no Rust destructors can be on the
/// stack. They will be skipped and not executed.
pub unsafe fn resume_panic(payload: Box<dyn Any + Send>) -> ! {
    tls::with(|info| info.unwrap().unwind_with(UnwindReason::Panic(payload)))
}

/// Stores trace message with backtrace.
#[derive(Debug)]
pub struct Trap {
    /// Original reason from where this trap originated.
    pub reason: TrapReason,
    /// Wasm backtrace of the trap, if any.
    pub backtrace: Option<Backtrace>,
    /// The Wasm Coredump, if any.
    pub coredumpstack: Option<CoreDumpStack>,
}

/// Enumeration of different methods of raising a trap.
#[derive(Debug)]
pub enum TrapReason {
    /// A user-raised trap through `raise_user_trap`.
    User {
        /// The actual user trap error.
        error: Error,
        /// Whether we need to capture a backtrace for this error or not.
        needs_backtrace: bool,
    },

    /// A trap raised from Cranelift-generated code.
    Jit {
        /// The program counter where this trap originated.
        ///
        /// This is later used with side tables from compilation to translate
        /// the trapping address to a trap code.
        pc: usize,

        /// If the trap was a memory-related trap such as SIGSEGV then this
        /// field will contain the address of the inaccessible data.
        ///
        /// Note that wasm loads/stores are not guaranteed to fill in this
        /// information. Dynamically-bounds-checked memories, for example, will
        /// not access an invalid address but may instead load from NULL or may
        /// explicitly jump to a `ud2` instruction. This is only available for
        /// fault-based traps which are one of the main ways, but not the only
        /// way, to run wasm.
        faulting_addr: Option<usize>,
    },

    /// A trap raised from a wasm libcall
    Wasm(wasmtime_environ::Trap),
}

impl TrapReason {
    /// Create a new `TrapReason::User` that does not have a backtrace yet.
    pub fn user_without_backtrace(error: Error) -> Self {
        TrapReason::User {
            error,
            needs_backtrace: true,
        }
    }

    /// Create a new `TrapReason::User` that already has a backtrace.
    pub fn user_with_backtrace(error: Error) -> Self {
        TrapReason::User {
            error,
            needs_backtrace: false,
        }
    }

    /// Is this a JIT trap?
    pub fn is_jit(&self) -> bool {
        matches!(self, TrapReason::Jit { .. })
    }
}

impl From<Error> for TrapReason {
    fn from(err: Error) -> Self {
        TrapReason::user_without_backtrace(err)
    }
}

impl From<wasmtime_environ::Trap> for TrapReason {
    fn from(code: wasmtime_environ::Trap) -> Self {
        TrapReason::Wasm(code)
    }
}

/// Catches any wasm traps that happen within the execution of `closure`,
/// returning them as a `Result`.
///
/// Highly unsafe since `closure` won't have any dtors run.
pub unsafe fn catch_traps<'a, F>(
    signal_handler: Option<*const SignalHandler<'static>>,
    capture_backtrace: bool,
    capture_coredump: bool,
    caller: *mut VMContext,
    mut closure: F,
) -> Result<(), Box<Trap>>
where
    F: FnMut(*mut VMContext),
{
    let limits = Instance::from_vmctx(caller, |i| i.runtime_limits());

    let result = CallThreadState::new(signal_handler, capture_backtrace, capture_coredump, *limits)
        .with(|cx| {
            traphandlers::wasmtime_setjmp(
                cx.jmp_buf.as_ptr(),
                call_closure::<F>,
                &mut closure as *mut F as *mut u8,
                caller,
            )
        });

    return match result {
        Ok(x) => Ok(x),
        Err((UnwindReason::Trap(reason), backtrace, coredumpstack)) => Err(Box::new(Trap {
            reason,
            backtrace,
            coredumpstack,
        })),
        Err((UnwindReason::Panic(panic), _, _)) => std::panic::resume_unwind(panic),
    };

    extern "C" fn call_closure<F>(payload: *mut u8, caller: *mut VMContext)
    where
        F: FnMut(*mut VMContext),
    {
        unsafe { (*(payload as *mut F))(caller) }
    }
}

// Module to hide visibility of the `CallThreadState::prev` field and force
// usage of its accessor methods.
mod call_thread_state {
    use super::*;

    /// Temporary state stored on the stack which is registered in the `tls` module
    /// below for calls into wasm.
    pub struct CallThreadState {
        pub(super) unwind:
            UnsafeCell<MaybeUninit<(UnwindReason, Option<Backtrace>, Option<CoreDumpStack>)>>,
        pub(super) jmp_buf: Cell<*const u8>,
        pub(super) signal_handler: Option<*const SignalHandler<'static>>,
        pub(super) capture_backtrace: bool,
        pub(super) capture_coredump: bool,

        pub(crate) limits: *const VMRuntimeLimits,

        pub(super) prev: Cell<tls::Ptr>,

        // The values of `VMRuntimeLimits::last_wasm_{exit_{pc,fp},entry_sp}`
        // for the *previous* `CallThreadState` for this same store/limits. Our
        // *current* last wasm PC/FP/SP are saved in `self.limits`. We save a
        // copy of the old registers here because the `VMRuntimeLimits`
        // typically doesn't change across nested calls into Wasm (i.e. they are
        // typically calls back into the same store and `self.limits ==
        // self.prev.limits`) and we must to maintain the list of
        // contiguous-Wasm-frames stack regions for backtracing purposes.
        old_last_wasm_exit_fp: Cell<usize>,
        old_last_wasm_exit_pc: Cell<usize>,
        old_last_wasm_entry_sp: Cell<usize>,
    }

    impl Drop for CallThreadState {
        fn drop(&mut self) {
            unsafe {
                *(*self.limits).last_wasm_exit_fp.get() = self.old_last_wasm_exit_fp.get();
                *(*self.limits).last_wasm_exit_pc.get() = self.old_last_wasm_exit_pc.get();
                *(*self.limits).last_wasm_entry_sp.get() = self.old_last_wasm_entry_sp.get();
            }
        }
    }

    impl CallThreadState {
        #[inline]
        pub(super) fn new(
            signal_handler: Option<*const SignalHandler<'static>>,
            capture_backtrace: bool,
            capture_coredump: bool,
            limits: *const VMRuntimeLimits,
        ) -> CallThreadState {
            CallThreadState {
                unwind: UnsafeCell::new(MaybeUninit::uninit()),
                jmp_buf: Cell::new(ptr::null()),
                signal_handler,
                capture_backtrace,
                capture_coredump,
                limits,
                prev: Cell::new(ptr::null()),
                old_last_wasm_exit_fp: Cell::new(unsafe { *(*limits).last_wasm_exit_fp.get() }),
                old_last_wasm_exit_pc: Cell::new(unsafe { *(*limits).last_wasm_exit_pc.get() }),
                old_last_wasm_entry_sp: Cell::new(unsafe { *(*limits).last_wasm_entry_sp.get() }),
            }
        }

        /// Get the saved FP upon exit from Wasm for the previous `CallThreadState`.
        pub fn old_last_wasm_exit_fp(&self) -> usize {
            self.old_last_wasm_exit_fp.get()
        }

        /// Get the saved PC upon exit from Wasm for the previous `CallThreadState`.
        pub fn old_last_wasm_exit_pc(&self) -> usize {
            self.old_last_wasm_exit_pc.get()
        }

        /// Get the saved SP upon entry into Wasm for the previous `CallThreadState`.
        pub fn old_last_wasm_entry_sp(&self) -> usize {
            self.old_last_wasm_entry_sp.get()
        }

        /// Get the previous `CallThreadState`.
        pub fn prev(&self) -> tls::Ptr {
            self.prev.get()
        }

        #[inline]
        pub(crate) unsafe fn push(&self) {
            assert!(self.prev.get().is_null());
            self.prev.set(tls::raw::replace(self));
        }

        #[inline]
        pub(crate) unsafe fn pop(&self) {
            let prev = self.prev.replace(ptr::null());
            let head = tls::raw::replace(prev);
            assert!(std::ptr::eq(head, self));
        }
    }
}
pub use call_thread_state::*;

enum UnwindReason {
    Panic(Box<dyn Any + Send>),
    Trap(TrapReason),
}

impl CallThreadState {
    #[inline]
    fn with(
        mut self,
        closure: impl FnOnce(&CallThreadState) -> i32,
    ) -> Result<(), (UnwindReason, Option<Backtrace>, Option<CoreDumpStack>)> {
        let ret = tls::set(&mut self, |me| closure(me));
        if ret != 0 {
            Ok(())
        } else {
            Err(unsafe { self.read_unwind() })
        }
    }

    #[cold]
    unsafe fn read_unwind(&self) -> (UnwindReason, Option<Backtrace>, Option<CoreDumpStack>) {
        (*self.unwind.get()).as_ptr().read()
    }

    fn unwind_with(&self, reason: UnwindReason) -> ! {
        let (backtrace, coredump) = match reason {
            // Panics don't need backtraces. There is nowhere to attach the
            // hypothetical backtrace to and it doesn't really make sense to try
            // in the first place since this is a Rust problem rather than a
            // Wasm problem.
            UnwindReason::Panic(_)
            // And if we are just propagating an existing trap that already has
            // a backtrace attached to it, then there is no need to capture a
            // new backtrace either.
            | UnwindReason::Trap(TrapReason::User {
                needs_backtrace: false,
                ..
            }) => (None, None),
            UnwindReason::Trap(_) => (self.capture_backtrace(self.limits, None), self.capture_coredump(self.limits, None)),
        };
        unsafe {
            (*self.unwind.get())
                .as_mut_ptr()
                .write((reason, backtrace, coredump));
            traphandlers::wasmtime_longjmp(self.jmp_buf.get());
        }
    }

    /// Trap handler using our thread-local state.
    ///
    /// * `pc` - the program counter the trap happened at
    /// * `call_handler` - a closure used to invoke the platform-specific
    ///   signal handler for each instance, if available.
    ///
    /// Attempts to handle the trap if it's a wasm trap. Returns a few
    /// different things:
    ///
    /// * null - the trap didn't look like a wasm trap and should continue as a
    ///   trap
    /// * 1 as a pointer - the trap was handled by a custom trap handler on an
    ///   instance, and the trap handler should quickly return.
    /// * a different pointer - a jmp_buf buffer to longjmp to, meaning that
    ///   the wasm trap was succesfully handled.
    #[cfg_attr(miri, allow(dead_code))] // miri doesn't handle traps yet
    pub(crate) fn take_jmp_buf_if_trap(
        &self,
        pc: *const u8,
        call_handler: impl Fn(&SignalHandler) -> bool,
    ) -> *const u8 {
        // If we haven't even started to handle traps yet, bail out.
        if self.jmp_buf.get().is_null() {
            return ptr::null();
        }

        // First up see if any instance registered has a custom trap handler,
        // in which case run them all. If anything handles the trap then we
        // return that the trap was handled.
        if let Some(handler) = self.signal_handler {
            if unsafe { call_handler(&*handler) } {
                return 1 as *const _;
            }
        }

        // If this fault wasn't in wasm code, then it's not our problem
        if unsafe { !IS_WASM_PC(pc as usize) } {
            return ptr::null();
        }

        // If all that passed then this is indeed a wasm trap, so return the
        // `jmp_buf` passed to `wasmtime_longjmp` to resume.
        self.take_jmp_buf()
    }

    pub(crate) fn take_jmp_buf(&self) -> *const u8 {
        self.jmp_buf.replace(ptr::null())
    }

    #[cfg_attr(miri, allow(dead_code))] // miri doesn't handle traps yet
    pub(crate) fn set_jit_trap(&self, pc: *const u8, fp: usize, faulting_addr: Option<usize>) {
        let backtrace = self.capture_backtrace(self.limits, Some((pc as usize, fp)));
        let coredump = self.capture_coredump(self.limits, Some((pc as usize, fp)));
        unsafe {
            (*self.unwind.get()).as_mut_ptr().write((
                UnwindReason::Trap(TrapReason::Jit {
                    pc: pc as usize,
                    faulting_addr,
                }),
                backtrace,
                coredump,
            ));
        }
    }

    fn capture_backtrace(
        &self,
        limits: *const VMRuntimeLimits,
        trap_pc_and_fp: Option<(usize, usize)>,
    ) -> Option<Backtrace> {
        if !self.capture_backtrace {
            return None;
        }

        Some(unsafe { Backtrace::new_with_trap_state(limits, self, trap_pc_and_fp) })
    }

    fn capture_coredump(
        &self,
        limits: *const VMRuntimeLimits,
        trap_pc_and_fp: Option<(usize, usize)>,
    ) -> Option<CoreDumpStack> {
        if !self.capture_coredump {
            return None;
        }
        Some(CoreDumpStack::new(&self, limits, trap_pc_and_fp))
    }

    pub(crate) fn iter<'a>(&'a self) -> impl Iterator<Item = &Self> + 'a {
        let mut state = Some(self);
        std::iter::from_fn(move || {
            let this = state?;
            state = unsafe { this.prev().as_ref() };
            Some(this)
        })
    }
}

struct ResetCell<'a, T: Copy>(&'a Cell<T>, T);

impl<T: Copy> Drop for ResetCell<'_, T> {
    #[inline]
    fn drop(&mut self) {
        self.0.set(self.1);
    }
}

// A private inner module for managing the TLS state that we require across
// calls in wasm. The WebAssembly code is called from C++ and then a trap may
// happen which requires us to read some contextual state to figure out what to
// do with the trap. This `tls` module is used to persist that information from
// the caller to the trap site.
pub(crate) mod tls {
    use super::CallThreadState;
    use std::mem;
    use std::ops::Range;

    pub use raw::Ptr;

    // An even *more* inner module for dealing with TLS. This actually has the
    // thread local variable and has functions to access the variable.
    //
    // Note that this is specially done to fully encapsulate that the accessors
    // for tls may or may not be inlined. Wasmtime's async support employs stack
    // switching which can resume execution on different OS threads. This means
    // that borrows of our TLS pointer must never live across accesses because
    // otherwise the access may be split across two threads and cause unsafety.
    //
    // This also means that extra care is taken by the runtime to save/restore
    // these TLS values when the runtime may have crossed threads.
    //
    // Note, though, that if async support is disabled at compile time then
    // these functions are free to be inlined.
    pub(super) mod raw {
        use super::CallThreadState;
        use std::cell::Cell;
        use std::ptr;

        pub type Ptr = *const CallThreadState;

        // The first entry here is the `Ptr` which is what's used as part of the
        // public interface of this module. The second entry is a boolean which
        // allows the runtime to perform per-thread initialization if necessary
        // for handling traps (e.g. setting up ports on macOS and sigaltstack on
        // Unix).
        thread_local!(static PTR: Cell<(Ptr, bool)> = const { Cell::new((ptr::null(), false)) });

        #[cfg_attr(feature = "async", inline(never))] // see module docs
        #[cfg_attr(not(feature = "async"), inline)]
        pub fn replace(val: Ptr) -> Ptr {
            PTR.with(|p| {
                // When a new value is configured that means that we may be
                // entering WebAssembly so check to see if this thread has
                // performed per-thread initialization for traps.
                let (prev, initialized) = p.get();
                if !initialized {
                    super::super::lazy_per_thread_init();
                }
                p.set((val, true));
                prev
            })
        }

        /// Eagerly initialize thread-local runtime functionality. This will be performed
        /// lazily by the runtime if users do not perform it eagerly.
        #[cfg_attr(feature = "async", inline(never))] // see module docs
        #[cfg_attr(not(feature = "async"), inline)]
        pub fn initialize() {
            PTR.with(|p| {
                let (state, initialized) = p.get();
                if initialized {
                    return;
                }
                super::super::lazy_per_thread_init();
                p.set((state, true));
            })
        }

        #[cfg_attr(feature = "async", inline(never))] // see module docs
        #[cfg_attr(not(feature = "async"), inline)]
        pub fn get() -> Ptr {
            PTR.with(|p| p.get().0)
        }
    }

    pub use raw::initialize as tls_eager_initialize;

    /// Opaque state used to persist the state of the `CallThreadState`
    /// activations associated with a fiber stack that's used as part of an
    /// async wasm call.
    pub struct AsyncWasmCallState {
        // The head of a linked list of activations that are currently present
        // on an async call's fiber stack. This pointer points to the oldest
        // activation frame where the `prev` links internally link to younger
        // activation frames.
        //
        // When pushed onto a thread this linked list is traversed to get pushed
        // onto the current thread at the time.
        state: raw::Ptr,
    }

    impl AsyncWasmCallState {
        /// Creates new state that initially starts as null.
        pub fn new() -> AsyncWasmCallState {
            AsyncWasmCallState {
                state: std::ptr::null_mut(),
            }
        }

        /// Pushes the saved state of this wasm's call onto the current thread's
        /// state.
        ///
        /// This will iterate over the linked list of states stored within
        /// `self` and push them sequentially onto the current thread's
        /// activation list.
        ///
        /// The returned `PreviousAsyncWasmCallState` captures the state of this
        /// thread just before this operation, and it must have its `restore`
        /// method called to restore the state when the async wasm is suspended
        /// from.
        ///
        /// # Unsafety
        ///
        /// Must be carefully coordinated with
        /// `PreviousAsyncWasmCallState::restore` and fiber switches to ensure
        /// that this doesn't push stale data and the data is popped
        /// appropriately.
        pub unsafe fn push(self) -> PreviousAsyncWasmCallState {
            // Our `state` pointer is a linked list of oldest-to-youngest so by
            // pushing in order of the list we restore the youngest-to-oldest
            // list as stored in the state of this current thread.
            let ret = PreviousAsyncWasmCallState { state: raw::get() };
            let mut ptr = self.state;
            while let Some(state) = ptr.as_ref() {
                ptr = state.prev.replace(std::ptr::null_mut());
                state.push();
            }
            ret
        }

        /// Performs a runtime check that this state is indeed null.
        pub fn assert_null(&self) {
            assert!(self.state.is_null());
        }

        /// Asserts that the current CallThreadState pointer, if present, is not
        /// in the `range` specified.
        ///
        /// This is used when exiting a future in Wasmtime to assert that the
        /// current CallThreadState pointer does not point within the stack
        /// we're leaving (e.g.  allocated for a fiber).
        pub fn assert_current_state_not_in_range(range: Range<usize>) {
            let p = raw::get() as usize;
            assert!(p < range.start || range.end < p);
        }
    }

    /// Opaque state used to help control TLS state across stack switches for
    /// async support.
    pub struct PreviousAsyncWasmCallState {
        // The head of a linked list, similar to the TLS state. Note though that
        // this list is stored in reverse order to assist with `push` and `pop`
        // below.
        //
        // After a `push` call this stores the previous head for the current
        // thread so we know when to stop popping during a `pop`.
        state: raw::Ptr,
    }

    impl PreviousAsyncWasmCallState {
        /// Pops a fiber's linked list of activations and stores them in
        /// `AsyncWasmCallState`.
        ///
        /// This will pop the top activation of this current thread continuously
        /// until it reaches whatever the current activation was when `push` was
        /// originally called.
        ///
        /// # Unsafety
        ///
        /// Must be paired with a `push` and only performed at a time when a
        /// fiber is being suspended.
        pub unsafe fn restore(self) -> AsyncWasmCallState {
            let thread_head = self.state;
            mem::forget(self);
            let mut ret = AsyncWasmCallState::new();
            loop {
                // If the current TLS state is as we originally found it, then
                // this loop is finished.
                let ptr = raw::get();
                if ptr == thread_head {
                    break ret;
                }

                // Pop this activation from the current thread's TLS state, and
                // then afterwards push it onto our own linked list within this
                // `AsyncWasmCallState`. Note that the linked list in `AsyncWasmCallState` is stored
                // in reverse order so a subsequent `push` later on pushes
                // everything in the right order.
                (*ptr).pop();
                if let Some(state) = ret.state.as_ref() {
                    (*ptr).prev.set(state);
                }
                ret.state = ptr;
            }
        }
    }

    impl Drop for PreviousAsyncWasmCallState {
        fn drop(&mut self) {
            panic!("must be consumed with `restore`");
        }
    }

    /// Configures thread local state such that for the duration of the
    /// execution of `closure` any call to `with` will yield `state`, unless
    /// this is recursively called again.
    #[inline]
    pub fn set<R>(state: &mut CallThreadState, closure: impl FnOnce(&CallThreadState) -> R) -> R {
        struct Reset<'a> {
            state: &'a CallThreadState,
        }

        impl Drop for Reset<'_> {
            #[inline]
            fn drop(&mut self) {
                unsafe {
                    self.state.pop();
                }
            }
        }

        unsafe {
            state.push();
            let reset = Reset { state };
            closure(reset.state)
        }
    }

    /// Returns the last pointer configured with `set` above, if any.
    pub fn with<R>(closure: impl FnOnce(Option<&CallThreadState>) -> R) -> R {
        let p = raw::get();
        unsafe { closure(if p.is_null() { None } else { Some(&*p) }) }
    }
}
