// A WORD OF CAUTION
//
// This entire file basically needs to be kept in sync with itself. It's not
// really possible to modify just one bit of this file without understanding
// all the other bits. Documentation tries to reference various bits here and
// there but try to make sure to read over everything before tweaking things!

use wasmtime_asm_macros::asm_func;

// fn(
//    top_of_stack(rdi): *mut u8
// )
//
// Switches to the parent of the stack identified by `top_of_stack`. This
// functions is only intended for the case where we have finished execution on
// the current stack and are returning to the parent.
// Thus, this function never returns.
asm_func!(
    "wasmtime_fibre_switch_to_parent",
    "
        // We need RDI later on, use RSI for top of stack instead.
        mov rsi, rdi

        mov rbp, -0x10[rsi]
        mov rsp, -0x18[rsi]

        // The stack_switch instruction uses register RDI for the payload.
        // Here, the payload indicates that we are returning (value 0).
        // See the test case at the end of this file to keep this in sync with
        // ControlEffect::return_()
        mov rdi, 0

        jmp -0x08[rsi]
    ",
);


// This is a pretty special function that has no real signature. Its use is to
// be the "base" function of all fibers. This entrypoint is used in
// `wasmtime_fibre_init` to bootstrap the execution of a new fiber.
//
// We also use this function as a persistent frame on the stack to emit dwarf
// information to unwind into the caller. This allows us to unwind from the
// fiber's stack back to the main stack that the fiber was called from. We use
// special dwarf directives here to do so since this is a pretty nonstandard
// function.
//
// If you're curious a decent introduction to CFI things and unwinding is at
// https://www.imperialviolet.org/2017/01/18/cfi.html
//
// Note that this function is never called directly. It is only ever entered
// when a `stack_switch` instruction loads its address when switching to a stack
// prepared by `FiberStack::initialize`.
//
// Executing `stack_switch` on a stack prepared by `FiberStack::initialize` as
// described in the comment on `FiberStack::initialize` leads to the following
// values in various registers when execution of wasmtime_fibre_start begins:
//
// RSP: TOS - 0x40
// RBP: TOS - 0x10
asm_func!(
    "wasmtime_fibre_start",
    "
        // TODO(frank-emrich): Restore DWARF information for this function. In
        // the meantime, debugging is possible using frame pointer walking.


        //
        // Note that the next 5 instructions amount to calling fiber_start
        // with the following arguments:
        // 1. TOS
        // 2. func_ref
        // 3. caller_vmctx
        // 4. args_ptr
        // 5. args_capacity
        //
        // Note that `fiber_start` never returns: Instead, it resume to the
        // parent using `wasmtime_fibre_switch_to_parent`.

        pop r8  // args_capacity
        pop rcx // args_ptr
        pop rdx // caller_vmctx
        pop rsi // func_ref
        lea rdi, 0x20[rsp] // TOS
        call {fiber_start}

        // We should never get here and purposely emit an invalid instruction.
        ud2
    ",
    fiber_start = sym super::fiber_start,
);


#[test]
fn test_return_payload() {
  // The following assumption is baked into `wasmtime_fibre_switch_to_parent`.
  assert_eq!(wasmtime_continuations::CONTROL_EFFECT_RETURN_DISCRIMINANT, 0);
}
