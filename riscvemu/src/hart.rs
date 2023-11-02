use memory::Memory;

use crate::{
    instr::decode::{Branch, DecodeError, Instr, Load, RegImm, RegReg, Store},
    mask,
};

use self::{
    memory::{ReadError, Wordsize},
    registers::Registers,
};
use std::mem;
use thiserror::Error;

pub mod memory;
pub mod registers;

/// RISC-V Hardware Thread
///
/// This is the simplest possible RISC-V hardware thread, which is an
/// execution environment interface where (see section 1.2 in the
/// specification):
///
/// * there is only one hart (this one), which supports only a single
///   privilege level (i.e. there is no notion of privilege)
/// * the hart implements only RV32I
/// * the initial state of the program is defined by a set of values
///   of memory and registers (including the program counter),
///   determined as part of making this object.
/// * all memory is readable and writable, and the full address space
///   is main memory (section 1.4)
/// * All required traps are fatal traps (section 1.6), causing this
///   execution environment (i.e. this single hart) to terminate.
///
/// The member function step() controls execution of the hart.  Each
/// time it is called, the instruction at the current PC is
/// executed. If an exception occurs, step() returns the trap type
/// that occurred, for the caller to take any action.  If step is
/// re-called, then the hart will continue to execute instructions
/// from its current state, which may or may not result in another
/// trap.
///
/// The default Hart has the memory, registers and pc all initialised
/// to zero.
#[derive(Debug, Default)]
pub struct Hart {
    pub pc: u32,
    pub registers: Registers,
    pub memory: Memory,
}

macro_rules! interpret_u32_as_signed {
    ($value:expr) => {{
        let signed: i32 = unsafe { mem::transmute($value) };
        signed
    }};
}

macro_rules! interpret_i32_as_unsigned {
    ($value:expr) => {{
        let unsigned: u32 = unsafe { mem::transmute($value) };
        unsigned
    }};
}

/// Take an unsigned value (u8, u16 or u32), and a bit position for the
/// sign bit, and copy the value of the sign bit into all the higher bits
/// of the u32.
fn sign_extend<T: Into<u32>>(value: T, sign_bit_position: u32) -> u32 {
    let value: u32 = value.into();
    let sign_bit = 1 & (value >> sign_bit_position);
    if sign_bit == 1 {
        let sign_extension = 0xffff_ffff - mask!(sign_bit_position);
        value | sign_extension
    } else {
        value
    }
}

/// Check that an address is aligned to a byte_boundary specified.
/// Return address-misaligned if not.
fn check_address_aligned(address: u32, byte_alignment: u32) -> Result<(), ExecutionError> {
    if address % 4 != 0 {
        // Section 2.2 intro of RISC-V unprivileged specification
        Err(ExecutionError::InstructionAddressMisaligned)
    } else {
        Ok(())
    }
}

/// Calculate the address of the next instruction by adding
/// four to the program counter (wrapping if necessary) and
/// returning the result
fn next_instruction_address(pc: u32) -> u32 {
    pc.wrapping_add(4)
}

/// Load upper immediate in 32-bit mode
///
/// Load the u_immediate into the upper 12 bits of the register
/// dest and fill the lower 20 bits with zeros. Set pc = pc + 4.
///
fn execute_lui_rv32i(hart: &mut Hart, dest: u8, u_immediate: u32) -> Result<(), ExecutionError> {
    hart.set_x(dest, u_immediate << 12)?;
    hart.increment_pc();
    Ok(())
}

/// Add upper immediate to program counter in 32-bit mode
///
/// Make a 32-bit value by setting its upper 12 bits to
/// u_immediate and its lower 20 bits to zero, and add
/// the current value of the program counter. Store the
/// result in the register dest. Set pc = pc + 4.
///
fn execute_auipc_rv32i(hart: &mut Hart, dest: u8, u_immediate: u32) -> Result<(), ExecutionError> {
    let value = hart.pc.wrapping_add(u_immediate << 12);
    hart.set_x(dest, value).unwrap();
    hart.increment_pc();
    Ok(())
}

/// Jump and link in 32-bit mode
///
/// Store the address of the next instruction (pc + 4) in
/// the register dest. Then set pc = pc + offset (an
/// unconditional jump relative to the program counter).
fn execute_jal_rv32i(hart: &mut Hart, dest: u8, offset: u32) -> Result<(), ExecutionError> {
    let return_address = next_instruction_address(hart.pc);
    hart.set_x(dest, return_address)?;
    let relative_address = sign_extend(offset, 20);
    hart.jump_relative_to_pc(relative_address)
}

/// Jump and link register in 32-bit mode
///
/// Store the address of the next instruction (pc + 4) in
/// the register dest. Then compute base + offset, set the
/// least significant bit to zero, and set the pc to the
/// result.
fn execute_jalr_rv32i(
    hart: &mut Hart,
    dest: u8,
    base: u8,
    offset: u16,
) -> Result<(), ExecutionError> {
    let return_address = next_instruction_address(hart.pc);
    hart.set_x(dest, return_address)?;
    let relative_address = sign_extend(offset, 11);
    let base_address = hart.x(base)?;
    let new_pc = 0xffff_fffe & base_address.wrapping_add(relative_address);
    hart.jump_to_address(new_pc)
}

/// Execute a conditional branch in 32-bit mode
///
/// Compute a condition specified by the mnemonic between the values
/// of the registers src1 and src2. If the result is false, do nothing.
/// Else, compute a pc-relative address by sign-extending offset, and
/// jump to that address, raising an address-misaligned exception if the
/// resulting program counter is not aligned to a 4-byte boundary.
fn execute_branch_rv32i(
    hart: &mut Hart,
    mnemonic: Branch,
    src1: u8,
    src2: u8,
    offset: u16,
) -> Result<(), ExecutionError> {
    let src1 = hart.x(src1)?;
    let src2 = hart.x(src2)?;
    let branch_taken = match mnemonic {
        Branch::Beq => src1 == src2,
        Branch::Bne => src1 != src2,
        Branch::Blt => {
            let src1: i32 = interpret_u32_as_signed!(src1);
            let src2: i32 = interpret_u32_as_signed!(src2);
            src1 < src2
        }
        Branch::Bge => {
            let src1: i32 = interpret_u32_as_signed!(src1);
            let src2: i32 = interpret_u32_as_signed!(src2);
            src1 >= src2
        }
        Branch::Bltu => src1 < src2,
        Branch::Bgeu => src1 >= src2,
    };

    if branch_taken {
        let relative_address = sign_extend(offset, 11);
        hart.jump_relative_to_pc(relative_address)
    } else {
        hart.increment_pc();
        Ok(())
    }
}

/// Execute a load instruction in 32-bit mode
///
/// Compute a load address by adding the register base to the
/// sign-extended offset, and load data at that address into
/// dest. The width of the loaded data, and whether to sign-
/// or zero-extend the result is determined by mnemonic.
fn execute_load_rv32i(
    hart: &mut Hart,
    mnemonic: Load,
    dest: u8,
    base: u8,
    offset: u16,
) -> Result<(), ExecutionError> {
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = match mnemonic {
        Load::Lb => sign_extend(
            u32::try_from(
                hart.memory
                    .read(load_address.into(), Wordsize::Byte)
                    .unwrap(),
            )
            .unwrap(),
            7,
        ),
        Load::Lh => sign_extend(
            u32::try_from(
                hart.memory
                    .read(load_address.into(), Wordsize::Halfword)
                    .unwrap(),
            )
            .unwrap(),
            15,
        ),
        Load::Lw => hart
            .memory
            .read(load_address.into(), Wordsize::Word)
            .unwrap()
            .try_into()
            .unwrap(),
        Load::Lbu => hart
            .memory
            .read(load_address.into(), Wordsize::Byte)
            .unwrap()
            .try_into()
            .unwrap(),
        Load::Lhu => hart
            .memory
            .read(load_address.into(), Wordsize::Halfword)
            .unwrap()
            .try_into()
            .unwrap(),
    };
    hart.set_x(dest, load_data)?;
    hart.increment_pc();
    Ok(())
}

/// Execute a store instruction in 32-bit mode
///
/// Compute a store address by adding the register base to the
/// sign-extended offset, and write data to that address from
/// dest. The width of the data to write is determined by the
/// mnemonic
fn execute_store_rv32i(
    hart: &mut Hart,
    mnemonic: Store,
    src: u8,
    base: u8,
    offset: u16,
) -> Result<(), ExecutionError> {
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = hart.x(src)?;
    match mnemonic {
        Store::Sb => hart
            .memory
            .write(store_address.into(), store_data.into(), Wordsize::Byte)
            .unwrap(),
        Store::Sh => hart
            .memory
            .write(store_address.into(), store_data.into(), Wordsize::Halfword)
            .unwrap(),
        Store::Sw => hart
            .memory
            .write(store_address.into(), store_data.into(), Wordsize::Word)
            .unwrap(),
    };
    hart.increment_pc();
    Ok(())
}

/// Execute a register-immediate operation in 32-bit mode
///
/// Compute an operation determined by the mnemonic between the
/// register src and the sign-extended value of i_immediate.
/// Place the result in the register dest.
fn execute_reg_imm_rv32i(
    hart: &mut Hart,
    mnemonic: RegImm,
    dest: u8,
    src: u8,
    i_immediate: u16,
) -> Result<(), ExecutionError> {
    let src: u32 = hart.x(src)?;
    let i_immediate = sign_extend(i_immediate, 11);
    let value = match mnemonic {
        RegImm::Addi => src.wrapping_add(i_immediate),
        RegImm::Slti => {
            let src: i32 = interpret_u32_as_signed!(src);
            let i_immediate: i32 = interpret_u32_as_signed!(i_immediate);
            (src < i_immediate) as u32
        }
        RegImm::Sltiu => (src < i_immediate) as u32,
        RegImm::Andi => src & i_immediate,
        RegImm::Ori => src | i_immediate,
        RegImm::Xori => src ^ i_immediate,
        RegImm::Slli => src << (0x1f & i_immediate),
        RegImm::Srli => src >> (0x1f & i_immediate),
        RegImm::Srai => {
            let src: i32 = interpret_u32_as_signed!(src);
            interpret_i32_as_unsigned!(src >> (0x1f & i_immediate))
        }
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

/// Execute a register-register operation in 32-bit mode
///
/// Compute an operation determined by the mnemonic between the
/// registers src1 and src2. Place the result in the register dest.
fn execute_reg_reg_rv32i(
    hart: &mut Hart,
    mnemonic: RegReg,
    dest: u8,
    src1: u8,
    src2: u8,
) -> Result<(), ExecutionError> {
    let src1: u32 = hart.x(src1)?;
    let src2: u32 = hart.x(src2)?;
    let value = match mnemonic {
        RegReg::Add => src1.wrapping_add(src2),
        RegReg::Sub => src1.wrapping_sub(src2),
        RegReg::Slt => {
            let src1: i32 = interpret_u32_as_signed!(src1);
            let src2: i32 = interpret_u32_as_signed!(src2);
            (src1 < src2) as u32
        }
        RegReg::Sltu => (src1 < src2) as u32,
        RegReg::And => src1 & src2,
        RegReg::Or => src1 | src2,
        RegReg::Xor => src1 ^ src2,
        RegReg::Sll => src1 << (0x1f & src2),
        RegReg::Srl => src1 >> (0x1f & src2),
        RegReg::Sra => {
            let src1: i32 = interpret_u32_as_signed!(src1);
            interpret_i32_as_unsigned!(src1 >> (0x1f & src2))
        }
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("encountered invalid register index {0}")]
    RegisterIndexInvalid(u8),
}

impl Hart {
    /// Read the value of the register xn
    pub fn x(&self, n: u8) -> Result<u32, RegisterError> {
        if n < 32 {
            let value: u32 = self
                .registers
                .read(n.into())
                .expect("index is valid, so no errors should occur on read")
                .try_into()
                .expect("only 32-bit values were written, so conversion should work");
            Ok(value)
        } else {
            Err(RegisterError::RegisterIndexInvalid(n))
        }
    }

    /// Write the value of the register xn
    pub fn set_x(&mut self, n: u8, value: u32) -> Result<(), RegisterError> {
        if n < 32 {
            self.registers
                .write(n.into(), value.into())
                .expect("index is valid, so no errors should occur on write");
            Ok(())
        } else {
            Err(RegisterError::RegisterIndexInvalid(n))
        }
    }

    /// Add 4 to the program counter, wrapping if necessary
    fn increment_pc(&mut self) {
        self.pc = next_instruction_address(self.pc);
    }

    /// Add an offset to the program counter, wrapping if necessary.
    /// If the resulting address is not aligned on a 4-byte boundary,
    /// return an address-misaligned exception (pc remains modified).
    fn jump_relative_to_pc(&mut self, offset: u32) -> Result<(), ExecutionError> {
        self.pc = self.pc.wrapping_add(offset);
        check_address_aligned(self.pc, 4)
    }

    /// Jump to a new instruction address (set pc = new_pc). Return
    /// an address-misaligned exception if the new_pc is not 4-byte
    /// aligned (pc remains modified).
    fn jump_to_address(&mut self, new_pc: u32) -> Result<(), ExecutionError> {
        self.pc = new_pc;
        check_address_aligned(self.pc, 4)
    }

    /// Get the program counter
    pub fn pc(&mut self) -> u32 {
        self.pc
    }

    pub fn read_memory(&mut self, address: u32, word_size: Wordsize) -> Result<u32, ExecutionError> {
        let value = self
            .memory
            .read(address.into(), word_size)?
            .try_into()
            .expect("the word should fit in u32, else bug in Memory");
        Ok(value)
    }
    
    fn execute(&mut self, instr: Instr) -> Result<(), ExecutionError> {
        match instr.clone() {
            Instr::Lui { dest, u_immediate } => execute_lui_rv32i(self, dest, u_immediate)?,
            Instr::Auipc { dest, u_immediate } => execute_auipc_rv32i(self, dest, u_immediate)?,
            Instr::Jal { dest, offset } => execute_jal_rv32i(self, dest, offset)?,
            Instr::Jalr { dest, base, offset } => execute_jalr_rv32i(self, dest, base, offset)?,
            Instr::Branch {
                mnemonic,
                src1,
                src2,
                offset,
            } => execute_branch_rv32i(self, mnemonic, src1, src2, offset)?,
            Instr::Load {
                mnemonic,
                dest,
                base,
                offset,
            } => execute_load_rv32i(self, mnemonic, dest, base, offset)?,
            Instr::Store {
                mnemonic,
                src,
                base,
                offset,
            } => execute_store_rv32i(self, mnemonic, src, base, offset)?,
            Instr::RegImm {
                mnemonic,
                dest,
                src,
                i_immediate,
            } => execute_reg_imm_rv32i(self, mnemonic, dest, src, i_immediate)?,
            Instr::RegReg {
                mnemonic,
                dest,
                src1,
                src2,
            } => execute_reg_reg_rv32i(self, mnemonic, dest, src1, src2)?,
        }
        Ok(())
    }

    /// Returns the instruction at the current program counter
    pub fn fetch_current_instruction(&mut self) -> u32 {
        self.read_memory(self.pc, Wordsize::Word)
            .expect("this read should succeed, else pc is invalid")
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        let instr = self.fetch_current_instruction();

        // Decoding the instruction may return traps, e.g. invalid
        // instruction.
        let instr = Instr::from(instr)?;

        // Execute instruction here. That may produce further traps,
        // e.g. ecalls or invalid instructions discovered at the
        // execution step
        self.execute(instr)?;

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum Trap {
    #[error("instruction decode failed: {0}")]
    InstructionDecodeFailed(DecodeError),
    #[error("instruction execution failed: {0}")]
    InstructionExecutionFailed(ExecutionError),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("invalid instruction {0:?}")]
    InvalidInstruction(Instr),
    #[error("instruction address should be aligned to a 4-byte boundary")]
    InstructionAddressMisaligned,
    #[error("register access error: {0}")]
    RegisterError(RegisterError),
    #[error("error occurred while reading memory: {0}")]
    MemoryReadError(ReadError),
}

impl From<RegisterError> for ExecutionError {
    fn from(e: RegisterError) -> ExecutionError {
        ExecutionError::RegisterError(e)
    }
}

impl From<ReadError> for ExecutionError {
    fn from(e: ReadError) -> ExecutionError {
        ExecutionError::MemoryReadError(e)
    }
}

impl From<DecodeError> for Trap {
    fn from(d: DecodeError) -> Trap {
        Trap::InstructionDecodeFailed(d)
    }
}

impl From<ExecutionError> for Trap {
    fn from(e: ExecutionError) -> Trap {
        Trap::InstructionExecutionFailed(e)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::instr::encode::*;

    #[test]
    fn check_lui() -> Result<(), &'static str> {
        // Check a basic case of lui (result should be placed in
        // upper 20 bits of x2)
        let mut hart = Hart::default();
        hart.memory
            .write(0, lui!(x2, 53).into(), Wordsize::Word)
            .unwrap();
        hart.step().unwrap();
        let x2 = hart.registers.read(2).unwrap();
        assert_eq!(x2, 53 << 12);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_auipc() -> Result<(), &'static str> {
        // Check a basic case of lui (result should be placed in
        // upper 20 bits of x2)
        let mut hart = Hart::default();
        hart.pc = 8;
        hart.memory
            .write(8, auipc!(x4, 53).into(), Wordsize::Word)
            .unwrap();
        hart.step().unwrap();
        let x4 = hart.registers.read(4).unwrap();
        assert_eq!(x4, 8 + (53 << 12));
        assert_eq!(hart.pc, 12);
        Ok(())
    }

    #[test]
    fn check_jal() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.pc = 8;
        hart.memory
            .write(8, jal!(x4, -4).into(), Wordsize::Word)
            .unwrap();
        hart.step().unwrap();
        let x4 = hart.registers.read(4).unwrap();
        assert_eq!(x4, 12);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_jalr() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.pc = 12;
        hart.registers.write(6, 20).unwrap();
        hart.memory
            .write(12, jalr!(x4, x6, -4).into(), Wordsize::Word)
            .unwrap();
        hart.step().unwrap();
        let x4 = hart.registers.read(4).unwrap();
        assert_eq!(x4, 16);
        assert_eq!(hart.pc, 20 - 4);
        Ok(())
    }

    #[test]
    fn check_beq_not_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, beq!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 1).unwrap();
        hart.registers.write(2, 2).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_beq_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, beq!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 2).unwrap();
        hart.registers.write(2, 2).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 16);
        Ok(())
    }

    #[test]
    fn check_bne_not_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bne!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 2).unwrap();
        hart.registers.write(2, 2).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_bne_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bne!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 1).unwrap();
        hart.registers.write(2, 2).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 16);
        Ok(())
    }

    #[test]
    fn check_blt_not_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, blt!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 10).unwrap();
        hart.registers.write(2, 0xffff_ffff).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_blt_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, blt!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 0xffff_ffff).unwrap();
        hart.registers.write(2, 10).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 16);
        Ok(())
    }

    #[test]
    fn check_bltu_not_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bltu!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 10).unwrap();
        hart.registers.write(2, 1).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_bltu_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bltu!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 1).unwrap();
        hart.registers.write(2, 10).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 16);
        Ok(())
    }

    #[test]
    fn check_bge_not_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bge!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 0xffff_ffff).unwrap();
        hart.registers.write(2, 10).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_bge_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bge!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 10).unwrap();
        hart.registers.write(2, 0xffff_ffff).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 16);
        Ok(())
    }

    #[test]
    fn check_bgeu_not_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bgeu!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 1).unwrap();
        hart.registers.write(2, 10).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_bgeu_taken() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, bgeu!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 10).unwrap();
        hart.registers.write(2, 1).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 16);
        Ok(())
    }

    #[test]
    fn check_lb() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, lb!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 4).unwrap();
        hart.memory.write(20, 0xff, Wordsize::Byte).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.registers.read(1).unwrap(), 0xffff_ffff);
        Ok(())
    }

    #[test]
    fn check_lbu() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, lbu!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 4).unwrap();
        hart.memory.write(20, 0xff, Wordsize::Byte).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.registers.read(1).unwrap(), 0x0000_00ff);
        Ok(())
    }

    #[test]
    fn check_lh() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, lh!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 5).unwrap();
        hart.memory.write(21, 0xff92, Wordsize::Halfword).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.registers.read(1).unwrap(), 0xffff_ff92);
        Ok(())
    }

    #[test]
    fn check_lhu() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, lhu!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 5).unwrap();
        hart.memory.write(21, 0xff92, Wordsize::Halfword).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.registers.read(1).unwrap(), 0x0000_ff92);
        Ok(())
    }

    #[test]
    fn check_lw() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, lw!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 6).unwrap();
        hart.memory.write(22, 0x1234_ff92, Wordsize::Word).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.registers.read(1).unwrap(), 0x1234_ff92);
        Ok(())
    }

    #[test]
    fn check_sb() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sb!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 0xfe).unwrap();
        hart.registers.write(2, 6).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.memory.read(22, Wordsize::Byte).unwrap(), 0xfe);
        Ok(())
    }

    #[test]
    fn check_sh() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sh!(x1, x2, 16).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 0xabfe).unwrap();
        hart.registers.write(2, 7).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.memory.read(23, Wordsize::Halfword).unwrap(), 0xabfe);
        Ok(())
    }

    #[test]
    fn check_sw() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sw!(x1, x2, -15).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(1, 0xabcd_ef12).unwrap();
        hart.registers.write(2, 20).unwrap();
        hart.step().unwrap();
        assert_eq!(hart.pc, 4);
        assert_eq!(hart.memory.read(5, Wordsize::Word).unwrap(), 0xabcd_ef12);
        Ok(())
    }

    #[test]
    fn check_addi() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, addi!(x1, x2, -23).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xffff_ffff);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_slti_both_positive() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slti!(x1, x2, 22).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 124).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, slti!(x1, x2, 124).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slti_both_negative() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slti!(x1, x2, -5).into(), Wordsize::Word)
            .unwrap();
        let v1: u64 = interpret_i32_as_unsigned!(-24).into();
        let v2: u64 = interpret_i32_as_unsigned!(-5).into();
        hart.registers.write(2, v1).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, slti!(x1, x2, -24).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, v2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slti_different_signs() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slti!(x1, x2, 5).into(), Wordsize::Word)
            .unwrap();
        let v1: u64 = interpret_i32_as_unsigned!(-24).into();
        let v2: u64 = 5;
        hart.registers.write(2, v1).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, slti!(x1, x2, -24).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, v2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_sltui() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sltiu!(x1, x2, 22).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 124).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, sltiu!(x1, x2, 124).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_andi() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, andi!(x1, x2, 0xff0).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x00ff_ff00).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        // Note that AND uses the sign-extended 12-bit immediate
        assert_eq!(x1, 0x00ff_ff00);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_ori() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, ori!(x1, x2, 0xff0).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x00ff_ff00).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xffff_fff0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_xori() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, xori!(x1, x2, 0xff0).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x00ff_ff00).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xff00_00f0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_slli() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slli!(x1, x2, 2).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0b1101).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0b110100);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_srli() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, srli!(x1, x2, 4).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xf000_0f00).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0x0f00_00f0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_srai() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, srai!(x1, x2, 4).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xf000_0f00).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xff00_00f0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_add() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, add!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 2).unwrap();
        hart.registers.write(3, 3).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 5);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_add_wrapping_edge_case() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, add!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xffff_fffe).unwrap();
        hart.registers.write(3, 5).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 3);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sub() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sub!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 124).unwrap();
        hart.registers.write(3, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 102);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sub_wrapping_edge_case() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sub!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 20).unwrap();
        hart.registers.write(3, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xffff_fffe);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_slt_both_positive() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slt!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 124).unwrap();
        hart.registers.write(3, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, slt!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(3, 124).unwrap();
        hart.registers.write(2, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slt_both_negative() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slt!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        let v1: u64 = interpret_i32_as_unsigned!(-24).into();
        let v2: u64 = interpret_i32_as_unsigned!(-5).into();
        hart.registers.write(2, v1).unwrap();
        hart.registers.write(3, v2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, slt!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(3, v1).unwrap();
        hart.registers.write(2, v2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slt_different_signs() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, slt!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        let v1: u64 = interpret_i32_as_unsigned!(-24).into();
        let v2: u64 = 5;
        hart.registers.write(2, v1).unwrap();
        hart.registers.write(3, v2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, slt!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(3, v1).unwrap();
        hart.registers.write(2, v2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_sltu() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sltu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 124).unwrap();
        hart.registers.write(3, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0);
        assert_eq!(hart.pc, 4);

        // Swap src1 and src2
        let mut hart = Hart::default();
        hart.memory
            .write(0, sltu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(3, 124).unwrap();
        hart.registers.write(2, 22).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);

        Ok(())
    }

    #[test]
    fn check_and() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, and!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x00ff_ff00).unwrap();
        hart.registers.write(3, 0x0f0f_f0f0).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0x000f_f000);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_or() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, or!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x00ff_ff00).unwrap();
        hart.registers.write(3, 0x0f0f_f0f0).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0x0fff_fff0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_xor() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, xor!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x00ff_ff00).unwrap();
        hart.registers.write(3, 0x0f0f_f0f0).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0x0ff0_0ff0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sll() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sll!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0b1101).unwrap();
        hart.registers.write(3, 2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0b110100);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_srl() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, srl!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xf000_0f00).unwrap();
        hart.registers.write(3, 4).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0x0f00_00f0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sra() -> Result<(), &'static str> {
        let mut hart = Hart::default();
        hart.memory
            .write(0, sra!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xf000_0f00).unwrap();
        hart.registers.write(3, 4).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xff00_00f0);
        assert_eq!(hart.pc, 4);
        Ok(())
    }
}
