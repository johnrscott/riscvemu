use memory::Memory;

use crate::instr::decode::{Instr, DecodeError};

use self::{memory::Wordsize, registers::Registers};
use thiserror::Error;
use std::mem;

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
	let signed: i32 = unsafe {mem::transmute($value)};
	signed
    }}
}

macro_rules! interpret_i32_as_unsigned {
    ($value:expr) => {{
	let unsigned: u32 = unsafe {mem::transmute($value)};
	unsigned
    }}
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
		self.registers.write(dest.into(), value.into()).unwrap();
		self.pc = self.pc.wrapping_add(offset.into());
		if self.pc % 4 != 0 {
		    // Section 2.2 intro of RISC-V unprivileged specification
		    return Err(ExecutionError::InstructionAddressMisaligned)
		}
	    }

	    Instr::RegReg { mnemonic, dest, src1, src2 } => {
		let src1: u32 = self.registers.read(src1.into()).unwrap().try_into().unwrap();
		let src2: u32 = self.registers.read(src2.into()).unwrap().try_into().unwrap();

		let value = match mnemonic.as_ref() {
		    "add" => src1.wrapping_add(src2),
		    "sub" => src1.wrapping_sub(src2), 
		    "slt" => {
			let src1: i32 = interpret_u32_as_signed!(src1);
			let src2: i32 = interpret_u32_as_signed!(src2);
			if src1 < src2 {1} else {0}
		    }, 
		    "sltu" => if src1 < src2 {1} else {0},
		    _ => return Err(ExecutionError::InvalidInstruction(instr))
		};
	
		self.registers.write(dest.into(), value.into()).unwrap();
		self.pc = self.pc.wrapping_add(4);

	    }
	    _ => return Err(ExecutionError::UnimplementedInstruction(instr))
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
    InstructionExecutionFailed(ExecutionError)
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
	hart.memory.write(0, lui!(x2, 53).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(8, auipc!(x4, 53).into(), Wordsize::Word).unwrap();
	hart.step().unwrap();
	let x4 = hart.registers.read(4).unwrap();
	assert_eq!(x4, 8 + (53 << 12));
	assert_eq!(hart.pc, 12);
	Ok(())
    }
    
    #[test]
    fn check_add() -> Result<(), &'static str> {
	let mut hart = Hart::default();
	hart.memory.write(0, add!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, add!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, sub!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, sub!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, slt!(x1, x2, x3).into(), Wordsize::Word).unwrap();
	hart.registers.write(2, 124).unwrap();
	hart.registers.write(3, 22).unwrap();
	hart.step().unwrap();
	let x1 = hart.registers.read(1).unwrap();
	assert_eq!(x1, 0);
	assert_eq!(hart.pc, 4);
	
	// Swap src1 and src2
	let mut hart = Hart::default();
	hart.memory.write(0, slt!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, slt!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, slt!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, slt!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, slt!(x1, x2, x3).into(), Wordsize::Word).unwrap();
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
	hart.memory.write(0, sltu!(x1, x2, x3).into(), Wordsize::Word).unwrap();
	hart.registers.write(2, 124).unwrap();
	hart.registers.write(3, 22).unwrap();
	hart.step().unwrap();
	let x1 = hart.registers.read(1).unwrap();
	assert_eq!(x1, 0);
	assert_eq!(hart.pc, 4);
	
	// Swap src1 and src2
	let mut hart = Hart::default();
	hart.memory.write(0, sltu!(x1, x2, x3).into(), Wordsize::Word).unwrap();
	hart.registers.write(3, 124).unwrap();
	hart.registers.write(2, 22).unwrap();
	hart.step().unwrap();
	let x1 = hart.registers.read(1).unwrap();
	assert_eq!(x1, 1);
	assert_eq!(hart.pc, 4);
	
	Ok(())
    }

}
