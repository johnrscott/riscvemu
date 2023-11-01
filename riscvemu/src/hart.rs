use memory::Memory;

use crate::{
    instr::decode::{DecodeError, Instr},
    mask,
};

use self::{memory::Wordsize, registers::Registers};
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

impl Hart {
    fn execute(&mut self, instr: Instr) -> Result<(), ExecutionError> {
        // Do something here depending on the instruction
        match instr.clone() {
            Instr::Lui { dest, u_immediate } => {
                let value = u_immediate << 12;
                self.registers.write(dest.into(), value.into()).unwrap();
                self.pc = self.pc.wrapping_add(4);
            }
            Instr::Auipc { dest, u_immediate } => {
                let value = self.pc.wrapping_add(u_immediate << 12);
                self.registers.write(dest.into(), value.into()).unwrap();
                self.pc = self.pc.wrapping_add(4);
            }
            Instr::Jal { dest, offset } => {
                let value = self.pc.wrapping_add(4);
                let offset = sign_extend(offset, 20);
                self.registers.write(dest.into(), value.into()).unwrap();
                self.pc = self.pc.wrapping_add(offset.into());
                if self.pc % 4 != 0 {
                    // Section 2.2 intro of RISC-V unprivileged specification
                    return Err(ExecutionError::InstructionAddressMisaligned);
                }
            }
            Instr::Jalr { dest, base, offset } => {
                let pc = self.pc.wrapping_add(4);
                self.registers.write(dest.into(), pc.into()).unwrap();
                let offset = sign_extend(offset, 11);
                let base: u32 = self
                    .registers
                    .read(base.into())
                    .unwrap()
                    .try_into()
                    .unwrap();
                self.pc = 0xffff_fffe & base.wrapping_add(offset);
                if self.pc % 4 != 0 {
                    // Section 2.2 intro of RISC-V unprivileged specification
                    return Err(ExecutionError::InstructionAddressMisaligned);
                }
            }
            Instr::Branch {
                mnemonic,
                src1,
                src2,
                offset,
            } => {
                let src1: u32 = self
                    .registers
                    .read(src1.into())
                    .unwrap()
                    .try_into()
                    .unwrap();
                let src2: u32 = self
                    .registers
                    .read(src2.into())
                    .unwrap()
                    .try_into()
                    .unwrap();

                let branch_taken = match mnemonic.as_ref() {
                    "beq" => src1 == src2,
                    "bne" => src1 != src2,
                    "blt" => {
                        let src1: i32 = interpret_u32_as_signed!(src1);
                        let src2: i32 = interpret_u32_as_signed!(src2);
                        src1 < src2
                    }
                    "bge" => {
                        let src1: i32 = interpret_u32_as_signed!(src1);
                        let src2: i32 = interpret_u32_as_signed!(src2);
                        src1 >= src2
                    }
                    "bltu" => src1 < src2,
                    "bgeu" => src1 >= src2,
                    _ => return Err(ExecutionError::InvalidInstruction(instr)),
                };

                if branch_taken {
                    let offset = sign_extend(offset, 11);
                    self.pc = self.pc.wrapping_add(offset.into());
                    if self.pc % 4 != 0 {
                        // Section 2.2 intro of RISC-V unprivileged specification
                        return Err(ExecutionError::InstructionAddressMisaligned);
                    }
                } else {
                    self.pc = self.pc.wrapping_add(4);
                }
            }
            Instr::Load {
                mnemonic,
                dest,
                base,
                offset,
            } => {
                let base: u32 = self
                    .registers
                    .read(base.into())
                    .unwrap()
                    .try_into()
                    .unwrap();
                let offset = sign_extend(offset, 11);
                let addr = base.wrapping_add(offset);
                let data = match mnemonic.as_ref() {
                    "lb" => sign_extend(
                        u32::try_from(self.memory.read(addr.into(), Wordsize::Byte).unwrap())
                            .unwrap(),
                        7,
                    ),
                    "lh" => sign_extend(
                        u32::try_from(self.memory.read(addr.into(), Wordsize::Halfword).unwrap())
                            .unwrap(),
                        15,
                    ),
                    "lw" => self
                        .memory
                        .read(addr.into(), Wordsize::Word)
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    "lbu" => self
                        .memory
                        .read(addr.into(), Wordsize::Byte)
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    "lhu" => self
                        .memory
                        .read(addr.into(), Wordsize::Halfword)
                        .unwrap()
                        .try_into()
                        .unwrap(),
                    _ => return Err(ExecutionError::InvalidInstruction(instr)),
                };
                self.registers.write(dest.into(), data.into()).unwrap();
                self.pc = self.pc.wrapping_add(4);
            }
            Instr::Store {
                mnemonic,
                src,
                base,
                offset,
            } => {
                let base: u32 = self
                    .registers
                    .read(base.into())
                    .unwrap()
                    .try_into()
                    .unwrap();
                let offset = sign_extend(offset, 11);
                let addr = base.wrapping_add(offset);
                let data: u32 = self.registers.read(src.into()).unwrap().try_into().unwrap();
                match mnemonic.as_ref() {
                    "sb" => self
                        .memory
                        .write(addr.into(), data.into(), Wordsize::Byte)
                        .unwrap(),
                    "sh" => self
                        .memory
                        .write(addr.into(), data.into(), Wordsize::Halfword)
                        .unwrap(),
                    "sw" => self
                        .memory
                        .write(addr.into(), data.into(), Wordsize::Word)
                        .unwrap(),
                    _ => return Err(ExecutionError::InvalidInstruction(instr)),
                };
                self.pc = self.pc.wrapping_add(4);
            }
            Instr::RegImm {
                mnemonic,
                dest,
                src,
                i_immediate,
            } => {
                let src: u32 = self.registers.read(src.into()).unwrap().try_into().unwrap();
                let i_immediate = sign_extend(i_immediate, 11);
                let value = match mnemonic.as_ref() {
                    "addi" => src.wrapping_add(i_immediate),
                    "slti" => {
                        let src: i32 = interpret_u32_as_signed!(src);
                        let i_immediate: i32 = interpret_u32_as_signed!(i_immediate);
                        (src < i_immediate) as u32
                    }
                    "sltiu" => (src < i_immediate) as u32,
                    "andi" => src & i_immediate,
                    "ori" => src | i_immediate,
                    "xori" => src ^ i_immediate,
                    "slli" => src << (0x1f & i_immediate),
                    "srli" => src >> (0x1f & i_immediate),
                    "srai" => {
                        let src: i32 = interpret_u32_as_signed!(src);
                        interpret_i32_as_unsigned!(src >> (0x1f & i_immediate))
                    }

                    _ => return Err(ExecutionError::InvalidInstruction(instr)),
                };
                self.registers.write(dest.into(), value.into()).unwrap();
                self.pc = self.pc.wrapping_add(4);
            }
            Instr::RegReg {
                mnemonic,
                dest,
                src1,
                src2,
            } => {
                let src1: u32 = self
                    .registers
                    .read(src1.into())
                    .unwrap()
                    .try_into()
                    .unwrap();
                let src2: u32 = self
                    .registers
                    .read(src2.into())
                    .unwrap()
                    .try_into()
                    .unwrap();

                let value = match mnemonic.as_ref() {
                    "add" => src1.wrapping_add(src2),
                    "sub" => src1.wrapping_sub(src2),
                    "slt" => {
                        let src1: i32 = interpret_u32_as_signed!(src1);
                        let src2: i32 = interpret_u32_as_signed!(src2);
                        (src1 < src2) as u32
                    }
                    "sltu" => (src1 < src2) as u32,
                    "and" => src1 & src2,
                    "or" => src1 | src2,
                    "xor" => src1 ^ src2,
                    "sll" => src1 << (0x1f & src2),
                    "srl" => src1 >> (0x1f & src2),
                    "sra" => {
                        let src1: i32 = interpret_u32_as_signed!(src1);
                        interpret_i32_as_unsigned!(src1 >> (0x1f & src2))
                    }
                    _ => return Err(ExecutionError::InvalidInstruction(instr)),
                };

                self.registers.write(dest.into(), value.into()).unwrap();
                self.pc = self.pc.wrapping_add(4);
            }
        }

        Ok(())
    }

    pub fn step(&mut self) -> Result<(), Trap> {
        // Fetch the instruction
        let instr: u32 = self
            .memory
            .read(self.pc.into(), Wordsize::Word)
            .expect("this read should succeed, else pc is invalid")
            .try_into()
            .expect("the word should fit in u32, else bug in Memory");

        // Decoding the instruction may return traps, e.g. invalid
        // instruction. That can be returned.
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
    #[error("invalid instruction")]
    InvalidInstruction(Instr),
    #[error("unimplemented instruction")]
    UnimplementedInstruction(Instr),
    #[error("instruction address should be aligned to a 4-byte boundary")]
    InstructionAddressMisaligned,
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
        assert_eq!(hart.pc, 12 + 20 - 4);
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