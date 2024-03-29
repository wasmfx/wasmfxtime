// A WORD OF CAUTION
//
// This entire file basically needs to be kept in sync with itself. It's not
// really possible to modify just one bit of this file without understanding
// all the other bits. Documentation tries to reference various bits here and
// there but try to make sure to read over everything before tweaking things!

use wasmtime_asm_macros::asm_func;

// fn(
//    top_of_stack(rdi): *mut u8
//    payload(rsi) : u64
// )
//
// The payload (i.e., second argument) is return unchanged, allowing data to be
// passed from the continuation that calls `wasmtime_fibre_switch` to the one
// that subsequently runs.
asm_func!(
    "wasmtime_fibre_switch",
    "
        // We're switching to arbitrary code somewhere else, so pessimistically
        // assume that all callee-save register are clobbered. This means we need
        // to save/restore all of them.
        //
        // Note that this order for saving is important since we use CFI directives
        // below to point to where all the saved registers are.
        //
        // The frame pointer must come first, so that we have the return address
        // and then the frame pointer on the stack (at addresses called 0xCFF8
        // and 0xCFF0, respectively, in the second picture in unix.rs)
        push rbp
        mov rbp, rsp
        push rbx // at -0x08[rbp]
        push r12 // at -0x10[rbp]
        push r13 // at -0x18[rbp]
        push r14 // at -0x20[rbp]
        push r15 // at -0x28[rbp]

        // Load the resume frame pointer that we're going to resume at and
        // store where we're going to get resumed from.
        mov rax, -0x10[rdi]
        mov -0x10[rdi], rbp


        // Swap stacks: We loaded the resume frame pointer into RAX, meaning
        // that it is near the beginning of the pseudo frame of the invocation of
        // wasmtime_fibe_switch that we want to get back to.
        // Thus, we need to turn this *frame* pointer back into the
        // corresponding *stack* pointer. This is simple: The resume frame
        // pointer is where wamtime_fibre_switch stored RBP, and we want to
        // calculate the stack pointer after it pushed the next 5 registers, too.
        //
        // Using the values from the second picture in unix.rs: If we loaded
        // 0xCFF0 into RAX, then we want to set RSP to 0xCFC8. Thus, to reflect
        // that an additional 5 registers where pushed on the stack after RBP, we
        // subtract 5 * 8 = 0x28 from RAX.
        lea rsp, -0x28[rax]
        // Restore callee-saved registers
        pop r15
        pop r14
        pop r13
        pop r12
        pop rbx
        pop rbp

        // We return the payload (i.e., the second argument to this function)
        mov rax, rsi

        ret
    ",
);

// fn(
//    top_of_stack(rdi): *mut u8,
//    func_ref(rsi): *const VMFuncRef,
//    caller_vmctx(rdx): *mut VMContext
//    args_ptr(rcx): *mut ValRaw
//    args_capacity(r8) : u64
//    wasmtime_fibre_switch_pc(r9): *mut u8,
// )
//
// This function installs the launchpad for the computation to run on the fiber,
// such that invoking wasmtime_fibre_switch on the stack actually runs the
// desired computation.
//
// Concretely, switching to the stack prepare by `wasmtime_fibre_init` function
// evokes that we enter `wasmtime_fibre_start`, which then in turn calls
// `fiber_start` with the arguments above.
//
// The layout of the FiberStack near the top of stack (TOS) *after* running this
// function is as follows:
//
//  Offset from    |
//       TOS       | Contents
//  ---------------|-----------------------------------------------------------
//          -0x08   wasmtime_fibre_switch_pc
//          -0x10   TOS - 0x20
//          -0x18   (RIP-relative) address of wasmtime_fibre_start function
//          -0x20   TOS - 0x10
//          -0x28   func_ref
//          -0x30   caller_vmctx
//          -0x38   args_ptr
//          -0x40   args_capacity
//          -0x48   undefined
#[rustfmt::skip]
asm_func!(
    "wasmtime_fibre_init",
    "
        // Here we're going to set up a stack frame as expected by
        // `wasmtime_fibre_switch`. The values we store here will get restored into
        // registers by that function and the `wasmtime_fibre_start` function will
        // take over and understands which values are in which registers.
        //
        // Install wasmtime_fibre_switch_pc at TOS - 0x08:
        mov -0x08[rdi], r9

        // Store TOS - 0x20 at TOS - 0x10
        // This is the resume frame pointer from which we calculate the new
        // value of RSP when switching to this stack.
        lea rax, -0x20[rdi]
        mov -0x10[rdi], rax // loaded first into rax during switch

        // Install wasmtime_fibre_start PC at TOS - 0x18
        lea r9, {start}[rip]
        mov -0x18[rdi], r9

        // Store TOS - 0x10 at TOS - 0x20
        // This is popped into RBP at the end of wasmtime_fibre_switch when
        // switching to this stack. It thus becomes the value of RBP while
        // executing wasmtime_fibre_start. Thus, wasmtime_fibre_start thinks
        // 'my parent's frame pointer is stored at TOS - 0x10'.
        // NB: RAX still contains TOS - 0x20 at this point.
        add rax, 0x10
        mov -0x20[rdi], rax

        // Install remaing arguments
        mov -0x28[rdi], rsi   // loaded into rbx during switch
        mov -0x30[rdi], rdx   // loaded into r12 during switch
        mov -0x38[rdi], rcx   // loaded into r13 during switch
        mov -0x40[rdi], r8    // loaded into r14 during switch

        ret
    ",
    start = sym super::wasmtime_fibre_start,
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
// Note that this function is never called directly. It is only ever entered via
// the return instruction in wasmtime_fibre_switch, with a stack that
// was prepared by wasmtime_fibre_init before calling wasmtime_fibre_switch.
//
// This execution of wasmtime_fibre_switch on a stack as described in the
// comment on wasmtime_fibre_init leads to the following values in various
// registers at the right before the RET instruction of the former is executed:
//
// RSP: TOS - 0x18
// RDI: TOS
// RSI: irrelevant  (not read by wasmtime_fibre_start)
// RAX: irrelevant  (not read by wasmtime_fibre_start)
// RBP: TOS - 0x10
// RBX: func_ref       (= VMFuncRef to execute)
// R12: caller_vmctx
// R13: args_ptr       (used by array call trampoline)
// R14: args_capacity  (used by array call trampoline)
// R15: irrelevant  (not read by wasmtime_fibre_start)
//
// At this point in time, the stack layout is as follows:
//
//  Offset from   |
//       TOS      | Contents
//  --------------|---------------------------------
//         -0x08   PC at beginning of wasmtime_fibre_switch
//
//         -0x10   frame pointer of wasmtime_fibre_switch that switched to us,
//                 thus pointing right below stack frame of caller of
//                 Fiber::resume, with pseudo frame of wasmtime_fibre_switch
//                 below.
//
//         -0x18   (RIP-relative) address of wasmtime_fibre_start function
//
//
// Note that after executing the RET instruction in wasmtime_fibre_switch,
// we then start executing wasmtime_fibre_start with RSP = TOS - 0x10.
asm_func!(
    "wasmtime_fibre_start",
    "
        // Use the `simple` directive on the startproc here which indicates that
        // some default settings for the platform are omitted, since this
        // function is so nonstandard
        .cfi_startproc simple
        .cfi_def_cfa_offset 0

        // This is where things get special, we're specifying a custom dwarf
        // expression for how to calculate the CFA. The goal here is that we
        // need to load the parent's stack pointer just before the call it made
        // into `wasmtime_fibre_switch`. Note that the CFA value changes over
        // time as well because a fiber may be resumed multiple times from
        // different points on the original stack. This means that our custom
        // CFA directive involves `DW_OP_deref`, which loads data from memory.
        //
        // The expression we're encoding here is that the CFA, the stack pointer
        // of whatever called into `wasmtime_fibre_start`, is:
        //
        //        *$rsp + 0x10
        //
        // $rsp is the stack pointer of `wasmtime_fibre_start` at the time the
        // next instruction after the `.cfi_escape` is executed. Our $rsp at the
        // start of this function is 16 bytes below the top of the stack (0xAff0
        // in the diagram in unix.rs). The $rbp of wasmtime_fibre_switch of our
        // parent invocation is stored at that location, so we dereference the
        // stack pointer to load it.
        //
        // After dereferencing, though, we have the $rbp value for
        // `wasmtime_fibre_switch` itself. That's a weird function which sort of
        // and sort of doesn't exist on the stack.  We want to point to the
        // caller of `wasmtime_fibre_switch`, so to do that we need to skip the
        // stack space reserved by `wasmtime_fibre_switch`, which is the saved
        // rbp register plus the return address of the caller's `call` instruction.
        // Hence we offset another 0x10 bytes.
        .cfi_escape 0x0f, /* DW_CFA_def_cfa_expression */ \
            4,            /* the byte length of this expression */ \
            0x57,         /* DW_OP_reg7 (rsp) */ \
            0x06,         /* DW_OP_deref */ \
            0x23, 0x10    /* DW_OP_plus_uconst 0x10 */

        // And now after we've indicated where our CFA is for our parent
        // function, we can define that where all of the saved registers are
        // located. This uses standard `.cfi` directives which indicate that
        // these registers are all stored relative to the CFA. Note that this
        // order is kept in sync with the above register spills in
        // `wasmtime_fibre_switch`.
        .cfi_rel_offset rip, -8
        .cfi_rel_offset rbp, -16
        .cfi_rel_offset rbx, -24
        .cfi_rel_offset r12, -32
        .cfi_rel_offset r13, -40
        .cfi_rel_offset r14, -48
        .cfi_rel_offset r15, -56

        // The body of this function is pretty similar. All our parameters are
        // already loaded into registers by the switch function. The
        // `wasmtime_fibre_init` routine arranged the various values to be
        // materialized into the registers used here. Our job is to then move
        // the values into the ABI-defined registers and call the entry-point
        // (i.e., the fiber_start function).
        // Note that `call` is used here to leave this frame on the stack so we
        // can use the dwarf info here for unwinding.
        //
        // Note that the next 5 instructions amount to calling fiber_start
        // with the following arguments:
        // 1. TOS
        // 2. func_ref
        // 3. caller_vmctx
        // 4. args_ptr
        // 5. args_capacity
        //
        // Note that fiber_start never returns: Instead, it // resume to the
        // parent FiberStack via wasmtime_fibre_switch.

        // TOS is already in RDI
        mov rsi, rbx // func_ref
        mov rdx, r12 // caller_vmctx
        mov rcx, r13 // args_ptr
        mov r8, r13  // args_capacity
        call {fiber_start}

        // We should never get here and purposely emit an invalid instruction.
        ud2
        .cfi_endproc
    ",
    fiber_start = sym super::fiber_start,
);
