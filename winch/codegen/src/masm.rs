use crate::abi::{self, align_to, LocalSlot};
use crate::codegen::{CodeGenContext, TableData};
use crate::isa::reg::Reg;
use cranelift_codegen::{Final, MachBufferFinalized, MachLabel};
use std::{fmt::Debug, ops::Range};
use wasmtime_environ::PtrSize;

pub(crate) use cranelift_codegen::ir::TrapCode;

#[derive(Eq, PartialEq)]
pub(crate) enum DivKind {
    /// Signed division.
    Signed,
    /// Unsigned division.
    Unsigned,
}

/// Remainder kind.
pub(crate) enum RemKind {
    /// Signed remainder.
    Signed,
    /// Unsigned remainder.
    Unsigned,
}

/// A stack slot.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct StackSlot {
    /// The location of the slot, relative to the stack pointer.
    pub offset: u32,
    /// The size of the slot, in bytes.
    pub size: u32,
}

/// Kinds of binary comparison in WebAssembly. The [`masm`] implementation for
/// each ISA is responsible for emitting the correct sequence of instructions
/// when lowering to machine code.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub(crate) enum CmpKind {
    /// Equal.
    Eq,
    /// Not equal.
    Ne,
    /// Signed less than.
    LtS,
    /// Unsigned less than.
    LtU,
    /// Signed greater than.
    GtS,
    /// Unsigned greater than.
    GtU,
    /// Signed less than or equal.
    LeS,
    /// Unsigned less than or equal.
    LeU,
    /// Signed greater than or equal.
    GeS,
    /// Unsigned greater than or equal.
    GeU,
}

/// Kinds of shifts in WebAssembly.The [`masm`] implementation for each ISA is
/// responsible for emitting the correct sequence of instructions when
/// lowering to machine code.
pub(crate) enum ShiftKind {
    /// Left shift.
    Shl,
    /// Signed right shift.
    ShrS,
    /// Unsigned right shift.
    ShrU,
    /// Left rotate.
    Rotl,
    /// Right rotate.
    Rotr,
}

/// Operand size, in bits.
#[derive(Copy, Debug, Clone, Eq, PartialEq)]
pub(crate) enum OperandSize {
    /// 32 bits.
    S32,
    /// 64 bits.
    S64,
    /// 128 bits.
    S128,
}

impl OperandSize {
    /// The number of bits in the operand.
    pub fn num_bits(&self) -> i32 {
        match self {
            OperandSize::S32 => 32,
            OperandSize::S64 => 64,
            OperandSize::S128 => 128,
        }
    }

    /// The number of bytes in the operand.
    pub fn bytes(&self) -> u32 {
        match self {
            Self::S32 => 4,
            Self::S64 => 8,
            Self::S128 => 16,
        }
    }

    /// The binary logarithm of the number of bits in the operand.
    pub fn log2(&self) -> u8 {
        match self {
            OperandSize::S32 => 5,
            OperandSize::S64 => 6,
            OperandSize::S128 => 7,
        }
    }

    /// Create an [`OperandSize`]  from the given number of bytes.
    pub fn from_bytes(bytes: u8) -> Self {
        use OperandSize::*;
        match bytes {
            4 => S32,
            8 => S64,
            16 => S128,
            _ => panic!("Invalid bytes {} for OperandSize", bytes),
        }
    }
}

/// An abstraction over a register or immediate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum RegImm {
    /// A register.
    Reg(Reg),
    /// A tagged immediate argument.
    Imm(Imm),
}

/// An tagged representation of an immediate.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub(crate) enum Imm {
    /// I32 immediate.
    I32(u32),
    /// I64 immediate.
    I64(u64),
    /// F32 immediate.
    F32(u32),
    /// F64 immediate.
    F64(u64),
}

impl Imm {
    /// Create a new I64 immediate.
    pub fn i64(val: i64) -> Self {
        Self::I64(val as u64)
    }

    /// Create a new I32 immediate.
    pub fn i32(val: i32) -> Self {
        Self::I32(val as u32)
    }

    /// Create a new F32 immediate.
    // Temporary until support for f32.const is added.
    #[allow(dead_code)]
    pub fn f32(bits: u32) -> Self {
        Self::F32(bits)
    }

    /// Create a new F64 immediate.
    // Temporary until support for f64.const is added.
    #[allow(dead_code)]
    pub fn f64(bits: u64) -> Self {
        Self::F64(bits)
    }

    /// Convert the immediate to i32, if possible.
    pub fn to_i32(&self) -> Option<i32> {
        match self {
            Self::I32(v) => Some(*v as i32),
            Self::I64(v) => i32::try_from(*v as i64).ok(),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub(crate) enum CalleeKind {
    /// A function call to a raw address.
    Indirect(Reg),
    /// A function call to a local function.
    Direct(u32),
}

impl CalleeKind {
    /// Creates a callee kind from a register.
    pub fn indirect(reg: Reg) -> Self {
        Self::Indirect(reg)
    }

    /// Creates a direct callee kind from a function index.
    pub fn direct(index: u32) -> Self {
        Self::Direct(index)
    }
}

impl RegImm {
    /// Register constructor.
    pub fn reg(r: Reg) -> Self {
        RegImm::Reg(r)
    }

    /// I64 immediate constructor.
    pub fn i64(val: i64) -> Self {
        RegImm::Imm(Imm::i64(val))
    }

    /// I32 immediate constructor.
    pub fn i32(val: i32) -> Self {
        RegImm::Imm(Imm::i32(val))
    }

    /// F32 immediate, stored using its bits representation.
    // Temporary until support for f32.const is added.
    #[allow(dead_code)]
    pub fn f32(bits: u32) -> Self {
        RegImm::Imm(Imm::f32(bits))
    }

    /// F64 immediate, stored using its bits representation.
    // Temporary until support for f64.const is added.
    #[allow(dead_code)]
    pub fn f64(bits: u64) -> Self {
        RegImm::Imm(Imm::f64(bits))
    }

    /// Get the underlying register of the operand,
    /// if it is one.
    pub fn get_reg(&self) -> Option<Reg> {
        match self {
            Self::Reg(r) => Some(*r),
            _ => None,
        }
    }
}

impl From<Reg> for RegImm {
    fn from(r: Reg) -> Self {
        Self::Reg(r)
    }
}

pub enum RoundingMode {
    Nearest,
    Up,
    Down,
    Zero,
}

/// Generic MacroAssembler interface used by the code generation.
///
/// The MacroAssembler trait aims to expose an interface, high-level enough,
/// so that each ISA can provide its own lowering to machine code. For example,
/// for WebAssembly operators that don't have a direct mapping to a machine
/// a instruction, the interface defines a signature matching the WebAssembly
/// operator, allowing each implementation to lower such operator entirely.
/// This approach attributes more responsibility to the MacroAssembler, but frees
/// the caller from concerning about assembling the right sequence of
/// instructions at the operator callsite.
///
/// The interface defaults to a three-argument form for binary operations;
/// this allows a natural mapping to instructions for RISC architectures,
/// that use three-argument form.
/// This approach allows for a more general interface that can be restricted
/// where needed, in the case of architectures that use a two-argument form.

pub(crate) trait MacroAssembler {
    /// The addressing mode.
    type Address: Copy;

    /// The pointer representation of the target ISA,
    /// used to access information from [`VMOffsets`].
    type Ptr: PtrSize;

    /// The ABI details of the target.
    type ABI: abi::ABI;

    /// Emit the function prologue.
    fn prologue(&mut self);

    /// Emit the function epilogue.
    fn epilogue(&mut self, locals_size: u32);

    /// Reserve stack space.
    fn reserve_stack(&mut self, bytes: u32);

    /// Free stack space.
    fn free_stack(&mut self, bytes: u32);

    /// Reset the stack pointer to the given offset;
    ///
    /// Used to reset the stack pointer to a given offset
    /// when dealing with unreachable code.
    fn reset_stack_pointer(&mut self, offset: u32);

    /// Get the address of a local slot.
    fn local_address(&mut self, local: &LocalSlot) -> Self::Address;

    /// Loads the address of the table element at a given index.
    /// Returns a register that contains address of the table element.
    fn table_elem_address(
        &mut self,
        index: Reg,
        size: OperandSize,
        table_data: &TableData,
        context: &mut CodeGenContext,
    ) -> Reg;

    /// Constructs an address with an offset that is relative to the
    /// current position of the stack pointer (e.g. [sp + (sp_offset -
    /// offset)].
    fn address_from_sp(&self, offset: u32) -> Self::Address;

    /// Constructs an address with an offset that is absolute to the
    /// current position of the stack pointer (e.g. [sp + offset].
    fn address_at_sp(&self, offset: u32) -> Self::Address;

    /// Alias for [`Self::address_at_reg`] using the VMContext register as
    /// a base. The VMContext register is derived from the ABI type that is
    /// associated to the MacroAssembler.
    fn address_at_vmctx(&self, offset: u32) -> Self::Address;

    /// Construct an address that is absolute to the current position
    /// of the given register.
    fn address_at_reg(&self, reg: Reg, offset: u32) -> Self::Address;

    /// Emit a function call to either a local or external function.
    fn call(&mut self, stack_args_size: u32, f: impl FnMut(&mut Self) -> CalleeKind) -> u32;

    /// Get stack pointer offset.
    fn sp_offset(&self) -> u32;

    /// Perform a stack store.
    fn store(&mut self, src: RegImm, dst: Self::Address, size: OperandSize);

    /// Perform a stack load.
    fn load(&mut self, src: Self::Address, dst: Reg, size: OperandSize);

    /// Alias for `MacroAssembler::load` with the operand size corresponding
    /// to the pointer size of the target.
    fn load_ptr(&mut self, src: Self::Address, dst: Reg);

    /// Pop a value from the machine stack into the given register.
    fn pop(&mut self, dst: Reg, size: OperandSize);

    /// Perform a move.
    fn mov(&mut self, src: RegImm, dst: Reg, size: OperandSize);

    /// Perform a conditional move.
    fn cmov(&mut self, src: Reg, dst: Reg, cc: CmpKind, size: OperandSize);

    /// Perform add operation.
    fn add(&mut self, dst: Reg, lhs: Reg, rhs: RegImm, size: OperandSize);

    /// Perform subtraction operation.
    fn sub(&mut self, dst: Reg, lhs: Reg, rhs: RegImm, size: OperandSize);

    /// Perform multiplication operation.
    fn mul(&mut self, dst: Reg, lhs: Reg, rhs: RegImm, size: OperandSize);

    /// Perform a floating point abs operation.
    fn float_abs(&mut self, dst: Reg, size: OperandSize);

    /// Perform a floating point negation operation.
    fn float_neg(&mut self, dst: Reg, size: OperandSize);

    /// Perform a floating point floor operation.
    fn float_round(&mut self, mode: RoundingMode, dst: Reg, src: RegImm, size: OperandSize);

    /// Perform logical and operation.
    fn and(&mut self, dst: Reg, lhs: Reg, rhs: RegImm, size: OperandSize);

    /// Perform logical or operation.
    fn or(&mut self, dst: Reg, lhs: Reg, rhs: RegImm, size: OperandSize);

    /// Perform logical exclusive or operation.
    fn xor(&mut self, dst: Reg, lhs: Reg, rhs: RegImm, size: OperandSize);

    /// Perform a shift operation.
    /// Shift is special in that some architectures have specific expectations
    /// regarding the location of the instruction arguments. To free the
    /// caller from having to deal with the architecture specific constraints
    /// we give this function access to the code generation context, allowing
    /// each implementation to decide the lowering path.
    fn shift(&mut self, context: &mut CodeGenContext, kind: ShiftKind, size: OperandSize);

    /// Perform division operation.
    /// Division is special in that some architectures have specific
    /// expectations regarding the location of the instruction
    /// arguments and regarding the location of the quotient /
    /// remainder. To free the caller from having to deal with the
    /// architecure specific contraints we give this function access
    /// to the code generation context, allowing each implementation
    /// to decide the lowering path.  For cases in which division is a
    /// unconstrained binary operation, the caller can decide to use
    /// the `CodeGenContext::i32_binop` or `CodeGenContext::i64_binop`
    /// functions.
    fn div(&mut self, context: &mut CodeGenContext, kind: DivKind, size: OperandSize);

    /// Calculate remainder.
    fn rem(&mut self, context: &mut CodeGenContext, kind: RemKind, size: OperandSize);

    /// Compare src and dst and put the result in dst.
    fn cmp(&mut self, src: RegImm, dest: Reg, size: OperandSize);

    /// Compare src and dst and put the result in dst.
    /// This function will potentially emit a series of instructions.
    fn cmp_with_set(&mut self, src: RegImm, dst: Reg, kind: CmpKind, size: OperandSize);

    /// Count the number of leading zeroes in src and put the result in dst.
    /// In x64, this will emit multiple instructions if the `has_lzcnt` flag is
    /// false.
    fn clz(&mut self, src: Reg, dst: Reg, size: OperandSize);

    /// Count the number of trailing zeroes in src and put the result in dst.
    /// In x64, this will emit multiple instructions if the `has_tzcnt` flag is
    /// false.
    fn ctz(&mut self, src: Reg, dst: Reg, size: OperandSize);

    /// Push the register to the stack, returning the stack slot metadata.
    // NB
    // The stack alignment should not be assumed after any call to `push`,
    // unless explicitly aligned otherwise.  Typically, stack alignment is
    // maintained at call sites and during the execution of
    // epilogues.
    fn push(&mut self, src: Reg, size: OperandSize) -> StackSlot;

    /// Finalize the assembly and return the result.
    fn finalize(self) -> MachBufferFinalized<Final>;

    /// Zero a particular register.
    fn zero(&mut self, reg: Reg);

    /// Count the number of 1 bits in src and put the result in dst. In x64,
    /// this will emit multiple instructions if the `has_popcnt` flag is false.
    fn popcnt(&mut self, context: &mut CodeGenContext, size: OperandSize);

    /// Zero a given memory range.
    ///
    /// The default implementation divides the given memory range
    /// into word-sized slots. Then it unrolls a series of store
    /// instructions, effectively assigning zero to each slot.
    fn zero_mem_range(&mut self, mem: &Range<u32>) {
        let word_size = <Self::ABI as abi::ABI>::word_bytes();
        if mem.is_empty() {
            return;
        }

        let start = if mem.start % word_size == 0 {
            mem.start
        } else {
            // Ensure that the start of the range is at least 4-byte aligned.
            assert!(mem.start % 4 == 0);
            let start = align_to(mem.start, word_size);
            let addr: Self::Address = self.local_address(&LocalSlot::i32(start));
            self.store(RegImm::i32(0), addr, OperandSize::S32);
            // Ensure that the new start of the range, is word-size aligned.
            assert!(start % word_size == 0);
            start
        };

        let end = align_to(mem.end, word_size);
        let slots = (end - start) / word_size;

        if slots == 1 {
            let slot = LocalSlot::i64(start + word_size);
            let addr: Self::Address = self.local_address(&slot);
            self.store(RegImm::i64(0), addr, OperandSize::S64);
        } else {
            // TODO
            // Add an upper bound to this generation;
            // given a considerably large amount of slots
            // this will be inefficient.
            let zero = <Self::ABI as abi::ABI>::scratch_reg();
            self.zero(zero);
            let zero = RegImm::reg(zero);

            for step in (start..end).into_iter().step_by(word_size as usize) {
                let slot = LocalSlot::i64(step + word_size);
                let addr: Self::Address = self.local_address(&slot);
                self.store(zero, addr, OperandSize::S64);
            }
        }
    }

    /// Generate a label.
    fn get_label(&mut self) -> MachLabel;

    /// Bind the given label at the current code offset.
    fn bind(&mut self, label: MachLabel);

    /// Conditional branch.
    ///
    /// Performs a comparison between the two operands,
    /// and immediately after emits a jump to the given
    /// label destination if the condition is met.
    fn branch(&mut self, kind: CmpKind, lhs: RegImm, rhs: Reg, taken: MachLabel, size: OperandSize);

    /// Emits and unconditional jump to the given label.
    fn jmp(&mut self, target: MachLabel);

    /// Emits a jump table sequence. The default label is specified as
    /// the last element of the targets slice.
    fn jmp_table(&mut self, targets: &[MachLabel], index: Reg, tmp: Reg);

    /// Emit an unreachable code trap.
    fn unreachable(&mut self);

    /// Traps if the condition code is met.
    fn trapif(&mut self, cc: CmpKind, code: TrapCode);
}
