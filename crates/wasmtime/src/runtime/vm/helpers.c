// When using _FORTIFY_SOURCE with `longjmp` causes longjmp_chk to be used
// instead. longjmp_chk ensures that the jump target is on the existing stack.
// For our use case of jumping between stacks we need to disable it.
#undef _FORTIFY_SOURCE

#include <setjmp.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

#if (defined(__GNUC__) && !defined(__clang__))
#define WASMTIME_GCC 1
#endif

#ifdef CFG_TARGET_OS_windows

// Windows is required to use normal `setjmp` and `longjmp`.
#define platform_setjmp(buf) setjmp(buf)
#define platform_longjmp(buf, arg) longjmp(buf, arg)
typedef jmp_buf platform_jmp_buf;

#elif defined(WASMTIME_GCC) || defined(__x86_64__)

// clang-format off

// GCC and Clang on x86_64 provide `__builtin_setjmp`/`__builtin_longjmp`, which
// differ from plain `setjmp` and `longjmp` in that they're implemented by
// the compiler inline rather than in libc, and the compiler can avoid saving
// and restoring most of the registers. See the [GCC docs] and [clang docs]
// for more information.
//
// Per the caveat in the GCC docs, this assumes that the host compiler (which
// may be compiling for a generic architecture family) knows about all the
// register state that Cranelift (which may be specializing for the hardware at
// runtime) is assuming is callee-saved.
//
// [GCC docs]: https://gcc.gnu.org/onlinedocs/gcc/Nonlocal-Gotos.html
// [clang docs]: https://llvm.org/docs/ExceptionHandling.html#llvm-eh-sjlj-setjmp

// clang-format on
#define platform_setjmp(buf) __builtin_setjmp(buf)
#define platform_longjmp(buf, arg) __builtin_longjmp(buf, arg)
typedef void *platform_jmp_buf[5]; // this is the documented size; see the docs
                                   // links for details.

#else

// All other platforms/compilers funnel in here.
//
// Note that `sigsetjmp` and `siglongjmp` are used here where possible to
// explicitly pass a 0 argument to `sigsetjmp` that we don't need to preserve
// the process signal mask. This should make this call a bit faster b/c it
// doesn't need to touch the kernel signal handling routines.
#define platform_setjmp(buf) sigsetjmp(buf, 0)
#define platform_longjmp(buf, arg) siglongjmp(buf, arg)
typedef sigjmp_buf platform_jmp_buf;

#endif

#define CONCAT2(a, b) a##b
#define CONCAT(a, b) CONCAT2(a, b)
#define VERSIONED_SYMBOL(a) CONCAT(a, VERSIONED_SUFFIX)

// Define one function here, `wasmtime_setjmp_inverted`, which returns the
// negation of whether the call succeeded. Define then the actual import below
// of `wasmtime_setjmp_*` which returns the negation of this negation which
// means it returns whether the function invocation succeeded or not.
//
// Why in the world would we do this? For now: MinGW. In
// bytecodealliance/wasmtime#9675 that PR was originally failing CI only on
// MinGW and seems to be fixed by this. In that PR the signature of `body` here
// changed from a `void` return to a `bool` returned. That means that the body
// of this function changed from the historical:
//
//    body(payload, callee);
//    return 1;
//
// to what we actually want:
//
//    return body(payload, callee);
//
// For some reason though this causes issues when unwinding via `longjmp` on
// Windows. Tests would exit with the error message:
//
//    code 0xc0000028: An invalid or unaligned stack was encountered during an
//    unwind operation. (os error 543)
//
// Debugging revealed that if this:
//
//    return body(payload, callee);
//
// were written as:
//
//    bool ret = body(payload, callee);
//    return ret;
//
// then the bug would be "fixed". This "fix" didn't work in release mode
// however, leading to the current fix. For whatever reason it seems that
// unwinding is broken if there's not code between the `body(...)` indirect
// call and the function return. The `!` here below, inverting the return value,
// is the source of that "code".
//
// Ideally this `*_inverted` shim would go away and get past CI. It's unclear
// whether we're dealing with a miscompile in GCC, bad unwinding information
// generated by Cranelift for JIT code, or what. For now "this seems to work"
// but we'll also be in the process of forwarding this to some other Windows
// folks to see better what's going on.
static bool wasmtime_setjmp_inverted(void **buf_storage,
                                     bool (*body)(void *, void *),
                                     void *payload, void *callee) {
  platform_jmp_buf buf;
  if (platform_setjmp(buf) != 0) {
    return true;
  }
  *buf_storage = &buf;
  return !body(payload, callee);
}

bool VERSIONED_SYMBOL(wasmtime_setjmp)(void **buf_storage,
                                       bool (*body)(void *, void *),
                                       void *payload, void *callee) {
  return !wasmtime_setjmp_inverted(buf_storage, body, payload, callee);
}

void VERSIONED_SYMBOL(wasmtime_longjmp)(void *JmpBuf) {
  platform_jmp_buf *buf = (platform_jmp_buf *)JmpBuf;
  platform_longjmp(*buf, 1);
}

#ifdef CFG_TARGET_OS_windows
// export required for external access.
__declspec(dllexport)
#else
// Note the `weak` linkage here, though, which is intended to let other code
// override this symbol if it's defined elsewhere, since this definition doesn't
// matter.
// Just in case cross-language LTO is enabled we set the `noinline` attribute
// and also try to have some sort of side effect in this function with a dummy
// `asm` statement.
__attribute__((weak, noinline))
#endif
    void __jit_debug_register_code() {
#ifndef CFG_TARGET_OS_windows
  __asm__("");
#endif
}

struct JITDescriptor {
  uint32_t version_;
  uint32_t action_flag_;
  void *relevant_entry_;
  void *first_entry_;
};

#ifdef CFG_TARGET_OS_windows
// export required for external access.
__declspec(dllexport)
#else
// Note the `weak` linkage here which is the same purpose as above. We want to
// let other runtimes be able to override this since our own definition isn't
// important.
__attribute__((weak))
#endif
    struct JITDescriptor __jit_debug_descriptor = {1, 0, NULL, NULL};

struct JITDescriptor *VERSIONED_SYMBOL(wasmtime_jit_debug_descriptor)() {
  return &__jit_debug_descriptor;
}

// For more information about this see `unix/unwind.rs` and the
// `using_libunwind` function. The basic idea is that weak symbols aren't stable
// in Rust so we use a bit of C to work around that.
#ifndef CFG_TARGET_OS_windows
__attribute__((weak)) extern void __unw_add_dynamic_fde();

bool VERSIONED_SYMBOL(wasmtime_using_libunwind)() {
  return __unw_add_dynamic_fde != NULL;
}
#endif
