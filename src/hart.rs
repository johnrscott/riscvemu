use memory::Memory;

use crate::instr::decode::{Instr, DecodeError};

use self::{memory::Wordsize, registers::Registers};
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



impl Hart {

    fn execute(&mut self, instr: Instr) -> Result<(), Trap> {

	// Do something here depending on the instruction
	match instr {
	    Instr::Lui { dest, u_immediate } => {
		let value = u_immediate << 12;
		self.registers.write(dest.into(), value.into()).unwrap();
		self.pc += 4;
	    }
	    _ => unimplemented!("Instruction {instr} is not yet implemented"),
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
    InstructionDecodeFailed(DecodeError)
}

impl From<DecodeError> for Trap {
    fn from(d: DecodeError) -> Trap {
	Trap::InstructionDecodeFailed(d)
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


}
