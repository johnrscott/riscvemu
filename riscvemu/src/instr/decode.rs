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

use super::fields::*;

use super::rv32i::{Rv32i, decoders};
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

pub struct Rtype {
    pub rs1: u8,
    pub rs2: u8,
    pub rd: u8,
}

pub struct Itype {
    pub rs1: u8,
    pub imm: u16,
    pub rd: u8,
}

pub struct SBtype {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u16,
}

pub struct UJtype {
    pub rd: u8,
    pub imm: u32,
}

pub fn decode_rtype(instr: u32) -> Rtype {
    Rtype {
        rs1: rs1!(instr),
        rs2: rs2!(instr),
        rd: rd!(instr),
    }
}

pub fn decode_itype(instr: u32) -> Itype {
    Itype {
        rs1: rs1!(instr),
        imm: imm_itype!(instr),
        rd: rd!(instr),
    }
}

pub fn decode_stype(instr: u32) -> SBtype {
    SBtype {
        rs1: rs1!(instr),
        rs2: rs2!(instr),
        imm: imm_stype!(instr),
    }
}

pub fn decode_btype(instr: u32) -> SBtype {
    SBtype {
        rs1: rs1!(instr),
        rs2: rs2!(instr),
        imm: imm_btype!(instr).try_into().unwrap(),
    }
}

pub fn decode_utype(instr: u32) -> UJtype {
    UJtype {
        rd: rd!(instr),
        imm: lui_u_immediate!(instr),
    }
}

pub fn decode_jtype(instr: u32) -> UJtype {
    UJtype {
        rd: rd!(instr),
        imm: jal_offset!(instr),
    }
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
    InvalidSignature{ signature: u32, opcode: u32 }
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

#[derive(Debug)]
pub enum SignatureDecoder {
    /// If the opcode determines the function directly, then only
    /// the decoder function for the instruction is required.
    DecoderFunction{decoder: fn(u32)->Rv32i},
    /// Required if the opcode does not determine the instruction,
    /// but the signature does. Maps signatures to the function that
    /// can decode the instruction
    SignatureMap{ signature_function: fn(u32) -> u32, signature_to_decoder: HashMap<u32, fn(u32)->Rv32i>} ,
}

/// The RISC-V instruction decoder
///
/// The decoder is capable of handing different instruction classes.
#[derive(Debug, Default)]
pub struct Decoder {
    opcode_to_decoder: HashMap<u32, SignatureDecoder>,
}

impl Decoder {
    pub fn new(base_isa: BaseIsa, isa_extensions: Vec<IsaExtension>) -> Self {
	
	let opcode_to_decoder = match base_isa {
	    BaseIsa::Rv32i => {
		decoders()
	    },
	    _ => unimplemented!("No other base ISA yet"),
	};
        Self { opcode_to_decoder }
    }

    /// Decode an instruction
    ///
    /// If the instruction is a valid encoding of one of the instructions
    /// in the decoder, return the decoded instruction. Otherwise, return
    /// an error.
    ///
    /// Decoding happens with the following steps:
    /// - read the opcode field first
    ///   - if the opcode field determines the instruction, decode and return
    ///   - else, use opcode to determine either funct3 or funct7 to read next
    /// - read funct3/funct7 if required
    ///   - if funct3/funct7 determines the isntruction, decode and return
    ///   - elsde read the other of funct7/funct3
    /// - sometimes, ad-hoc actions are required to decode instruction, e,g,
    ///   reading high bits in srli and srai. Use these to decode and return
    ///   if necessary.
    ///
    /// Requirements:
    /// - the decoder should not attempt to decode ISA-by-ISA, because multiple
    ///   different ISAs can share an opcode (e.g. rv32i and rv32m share OP).
    ///   The decoding process for different ISAs should be merged (i.e. all of
    ///   the same opcode should be considered together)
    /// - The rules for how to decode should be stored in the ISAs themselves,
    ///   not the decoder.
    pub fn decode(&self, instr: u32) -> Result<Instr32, DecodeError> {

	// First get the opcode
	let opcode = opcode!(instr);
	if let Some(signature_decoder) = self.opcode_to_decoder.get(&opcode) {

	    // Use the opcode to determine whether a signature is required
	    match signature_decoder {
		SignatureDecoder::DecoderFunction { decoder } => Ok(decoder(instr).into()),
		SignatureDecoder::SignatureMap { signature_function, signature_to_decoder } => {
		    let signature = signature_function(instr);
		    if let Some(decoder) = signature_to_decoder.get(&signature) {
			Ok(decoder(instr).into())
		    } else {
			Err(DecodeError::InvalidSignature { signature, opcode })
		    }
		}
	    }   
	} else {
	    Err(DecodeError::InvalidOpcode(opcode))
	}
    }
}
