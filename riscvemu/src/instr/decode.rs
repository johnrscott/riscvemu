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
use std::collections::HashMap;

use crate::hart::{ExecutionError, Hart};

use super::fields::*;

use super::rv32i::{decoders, Rv32i};
use thiserror::Error;

// A signature will mean the value of an instruction with all
// non-opcoode fields (e.g. opcode, funct3 or funct7) zeroed out.
// By masking out non-opcode fields, the instruction can be
// determined by comparing with the signature. U- and J-types
// do not need signatures because the opcode already determines
// the instruction

pub fn rtype_signature(opcode: u32, funct3: u32, funct7: u32) -> u32 {
    funct7 << 25 | funct3 << 12 | opcode
}

pub fn isbtype_signature(opcode: u32, funct3: u32) -> u32 {
    funct3 << 12 | opcode
}

// Masking an instruction means setting all the non-signature fields
// to zero. This leaves it in a form that may be compared with the
// signature to determine what instruction is present. This comparison
// requires only one u32 operation. The correct signature to use may
// be obtained by reading the opcode field.

pub fn mask_rtype(instr: u32) -> u32 {
    (mask!(7) << 25 | mask!(3) << 12 | mask!(7)) & instr
}

pub fn mask_isbtype(instr: u32) -> u32 {
    (mask!(3) << 12 | mask!(7)) & instr
}

// /// Stores the functions required to decode an instruction
// pub struct DecodeFunctions {
//     /// Call this function to decode the non-opcode fields
//     /// of the instruction
//     decode: fn(u32) -> Rv32i,
// }

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("got invalid or unimplemented opcode 0x{0:x}")]
    InvalidOpcode(u32),
    #[error("got invalid or unimplemented instruction 0x{0:x}")]
    InvalidInstruction(u32),
    #[error("got invalid signature 0x{signature:b} for opcode 0x{opcode:b}")]
    InvalidSignature { signature: u32, opcode: u32 },
    #[error("instruction 0x{0:x} is missing an execution function")]
    MissingExecFunction(u32),
    #[error("instruction 0x{0:x} signature decoder is missing a map entry")]
    MissingValueInMap(u32),

      
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

#[derive(Debug, Default)]
pub enum BaseIsa {
    /// Base integer ISA, 32-bit
    #[default]
    Rv32i,
    /// Base integer ISA, 64-bit
    Rv64i,
}

pub enum IsaExtension {
    Rv32m,
    Rv64m,
}

/// An enum that holds every type of instruction defined in the
/// program. Could try to restrict this to just the ISAs which are
/// expected, but for now can leave it holding everything. (The main
/// purposes of splitting up the decoder is extensibility, not
/// optimisation.)
pub enum Instr {
    Rv32i(Rv32i),
}



// pub enum SignatureDecoder {
//     /// If the opcode determines the function directly, then only
//     /// the decoder function for the instruction is required.
//     DecoderFunction { decoder: fn(u32) -> Rv32i },
//     /// Required if the opcode does not determine the instruction,
//     /// but the signature does. Maps signatures to the function that
//     /// can decode the instruction
//     SignatureMap {
//         signature_function: fn(u32) -> u32,
//         signature_to_decoder: HashMap<u32, fn(u32) -> Rv32i>,
//     },
// }

// The decode process is as follows:
//
// 1. start with an instruction instr (u32). Set x = instr.
// 2. Set mask = opcode_mask (extracts the opcode)
// 3. apply mask to x, and compare the result to a set of values
// 4. depending on the value found, either read a new (value-specific) mask,
//    and go back to step 3; or, return a function which will execute the
//    instruction in 32-bit or 64-bit mode.
//

struct ExecFn32(fn(&mut Hart, instr: u32) -> Result<(), ExecutionError>);

/// This is a tree, containing a sequence of steps to decode an instruction
#[derive(Debug, Clone)]
pub enum SignatureDecoder {
    /// Variant for when another step of decoding is needed. 
    Decoder {
	/// For the next decoding step, use this mask
	next_mask: u32,
	/// Then compare the value you get to this map to
	/// obtain the next decoding step
	value_map: HashMap<u32, SignatureDecoder>
    },
    /// This is the leaf node, when the instruction is known
    Executer {
	xlen32_fn: Option<ExecFn32>,
    }
}

impl SignatureDecoder {
    fn get_exec_fn(&self, instr: u32) -> Result<ExecFn32, DecodeError> {
	match self {
	    Self::Decoder { next_mask, value_map } => {
		let value = next_mask & instr;
		if let Some(next_decoder) = value_map.get(&value) {
		    next_decoder.get_exec_fn(instr)
		} else {
		    Err(DecodeError::MissingValueInMap(instr))
		}
	    }
	    Self::Executer { xlen32_fn } => {
		if let Some(exec_fn) = xlen32_fn {
		    Ok(exec_fn)
		} else {
		    Err(DecodeError::MissingExecFunction)
		}
	    }
	}
    }

}

/// The RISC-V instruction decoder
///
/// The decoder is capable of handing different instruction classes.
#[derive(Debug, Default)]
pub struct Decoder {
    signature_decoder: SignatureDecoder
}

impl Decoder {
    pub fn new() -> Self {
	let mut signature_decoder = SignatureDecoder::Executer { xlen32_fn: () };
        Self { signature_decoder }
    }

    /// Decode an instruction
    pub fn decode(&self, instr: u32) -> Result<ExecFn32, DecodeError> {
        // First get the opcode
        let mut value = opcode!(instr);

	
    }
}
