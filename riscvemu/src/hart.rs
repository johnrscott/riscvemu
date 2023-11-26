use self::{
    memory::{ReadError, Wordsize},
    registers::Registers,
};
use crate::{
    decode::{Decoder, DecoderError},
    rv32i::{make_rv32i, Exec32},
};
use crate::{
    elf_utils::{ElfLoadError, ElfLoadable},
    fields::mask,
};
use memory::Memory;
use thiserror::Error;

pub mod csr;
pub mod machine;
pub mod memory;
pub mod platform;
pub mod pma;
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
#[derive(Debug)]
pub struct Hart {
    decoder: Decoder<Exec32>,
    pub pc: u32,
    pub registers: Registers,
    pub memory: Memory,
}

impl Default for Hart {
    /// The default hart implements the RV32I base instructions
    fn default() -> Self {
        let mut hart = Self {
            decoder: Decoder::new(mask!(7)),
            pc: 0,
            registers: Registers::default(),
            memory: Memory::default(),
        };

        make_rv32i(&mut hart.decoder).expect("adding these instructions should work");
        hart
    }
}

impl ElfLoadable for Hart {
    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), ElfLoadError> {
        self.memory
            .write(addr.into(), data.into(), Wordsize::Byte)
            .unwrap();
        Ok(())
    }
}

impl Hart {
    pub fn new(decoder: Decoder<Exec32>) -> Self {
        Self {
            decoder,
            ..Self::default()
        }
    }

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
    pub fn increment_pc(&mut self) {
        self.pc = next_instruction_address(self.pc);
    }

    /// Add an offset to the program counter, wrapping if necessary.
    /// If the resulting address is not aligned on a 4-byte boundary,
    /// return an address-misaligned exception (pc remains modified).
    pub fn jump_relative_to_pc(&mut self, offset: u32) -> Result<(), ExecutionError> {
        self.pc = self.pc.wrapping_add(offset);
        check_address_aligned(self.pc, 4)
    }

    /// Jump to a new instruction address (set pc = new_pc). Return
    /// an address-misaligned exception if the new_pc is not 4-byte
    /// aligned (pc remains modified).
    pub fn jump_to_address(&mut self, new_pc: u32) -> Result<(), ExecutionError> {
        self.pc = new_pc;
        check_address_aligned(self.pc, 4)
    }

    /// Get the program counter
    pub fn pc(&mut self) -> u32 {
        self.pc
    }

    pub fn read_memory(
        &mut self,
        address: u32,
        word_size: Wordsize,
    ) -> Result<u32, ExecutionError> {
        let value = self
            .memory
            .read(address.into(), word_size)?
            .try_into()
            .expect("the word should fit in u32, else bug in Memory");
        Ok(value)
    }

    /// Returns the instruction at the current program counter
    pub fn fetch_current_instruction(&mut self) -> u32 {
        self.read_memory(self.pc, Wordsize::Word)
            .expect("this read should succeed, else pc is invalid")
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        let instr = self.fetch_current_instruction();
        let exec_fn = self.decoder.get_exec(instr)?;
        exec_fn(self, instr)?;
        Ok(())
    }
}

/// Calculate the address of the next instruction by adding
/// four to the program counter (wrapping if necessary) and
/// returning the result
pub fn next_instruction_address(pc: u32) -> u32 {
    pc.wrapping_add(4)
}

/// Check that an address is aligned to a byte_boundary specified.
/// Return address-misaligned if not.
pub fn check_address_aligned(address: u32, byte_alignment: u32) -> Result<(), ExecutionError> {
    if address % byte_alignment != 0 {
        // Section 2.2 intro of RISC-V unprivileged specification
        Err(ExecutionError::InstructionAddressMisaligned)
    } else {
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum RegisterError {
    #[error("encountered invalid register index {0}")]
    RegisterIndexInvalid(u8),
}

#[derive(Error, Debug)]
pub enum Trap {
    #[error("instruction decode failed: {0}")]
    InstructionDecodeFailed(DecoderError),
    #[error("instruction execution failed: {0}")]
    InstructionExecutionFailed(ExecutionError),
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("invalid instruction {0:?}")]
    InvalidInstruction(u32),
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

impl From<DecoderError> for Trap {
    fn from(d: DecoderError) -> Trap {
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
    use crate::{encode::*, rv32m::make_rv32m};

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

    fn rv32im_hart() -> Hart {
        let mut decoder = Decoder::default();
        make_rv32i(&mut decoder).expect("adding instructions should work");
        make_rv32m(&mut decoder).expect("adding instructions should work");
        Hart::new(decoder)
    }

    #[test]
    fn check_mul() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, mul!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 5).unwrap();
        hart.registers
            .write(3, interpret_i32_as_unsigned!(-4).into())
            .unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, interpret_i32_as_unsigned!(-20).into());
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulh_positive() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, mulh!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0x7fff_ffff).unwrap();
        hart.registers.write(3, 4).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulh_negative() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, mulh!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xffff_ffff).unwrap();
        hart.registers.write(3, 4).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xffff_ffff);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulhu() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, mulhu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xffff_ffff).unwrap();
        hart.registers.write(3, 4).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 3);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulhsu_1() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, mulhsu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xffff_ffff).unwrap();
        hart.registers.write(3, 4).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0xffff_ffff);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulhsu_2() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, mulhsu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 4).unwrap();
        hart.registers.write(3, 0xffff_ffff).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 3);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_div() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, div!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 6).unwrap();
        hart.registers
            .write(3, interpret_i32_as_unsigned!(-3).into())
            .unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, interpret_i32_as_unsigned!(-2).into());
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_div_round_towards_zero() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, div!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 10).unwrap();
        hart.registers
            .write(3, interpret_i32_as_unsigned!(-3).into())
            .unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, interpret_i32_as_unsigned!(-3).into());
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_divu() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, divu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xe000_0000).unwrap();
        hart.registers.write(3, 2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 0x7000_0000);
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_rem() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, rem!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers
            .write(2, interpret_i32_as_unsigned!(-10).into())
            .unwrap();
        hart.registers.write(3, 3).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, interpret_i32_as_unsigned!(-1).into());
        assert_eq!(hart.pc, 4);
        Ok(())
    }

    #[test]
    fn check_remu() -> Result<(), &'static str> {
        let mut hart = rv32im_hart();
        hart.memory
            .write(0, remu!(x1, x2, x3).into(), Wordsize::Word)
            .unwrap();
        hart.registers.write(2, 0xe000_0003).unwrap();
        hart.registers.write(3, 2).unwrap();
        hart.step().unwrap();
        let x1 = hart.registers.read(1).unwrap();
        assert_eq!(x1, 1);
        assert_eq!(hart.pc, 4);
        Ok(())
    }
}
