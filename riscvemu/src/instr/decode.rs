//! Instruction Decoding
//!
//! This file is where a u32 instruction word is converted into
//! the Instr struct which holds the instruction type and fields
//! in a more easily accessible format ready for execution.
//!
//! v20191213, section 2.2: the behaviour upon decoding a reserved
//! instruction is unspecified. Specific behaviour for reserved
//! fields in instructions is documented where it occurs below.
//!
use super::rv32i::Rv32i;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("got invalid or unimplemented opcode 0x{0:x}")]
    InvalidOpcode(u32),
}

#[derive(Debug, Clone)]
pub enum Instr32 {
    Rv32i(Rv32i),
}

impl From<Rv32i> for Instr32 {
    fn from(rv32i_instr: Rv32i) -> Self {
	Self::Rv32i(rv32i_instr)
    }
}

impl Instr32 {

    /// Decode all the instruction extensions here
    pub fn from(instr: u32) -> Result<Self, DecodeError> {
	let rv32i_instr = Rv32i::from(instr)?;
	Ok(rv32i_instr.into())
    }
}
