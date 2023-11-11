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

//use super::rv32i::{decoders, Rv32i};
use thiserror::Error;

// A signature will mean the value of an instruction with all
// non-opcoode fields (e.g. opcode, funct3 or funct7) zeroed out.
// By masking out non-opcode fields, the instruction can be
// determined by comparing with the signature. U- and J-types
// do not need signatures because the opcode already determines
// the instruction

pub fn rtype_signature(funct3: u32, funct7: u32) -> u32 {
    funct7 << 25 | funct3 << 12
}

pub fn isbtype_signature(funct3: u32) -> u32 {
    funct3 << 12
}

// Masking an instruction means setting all the non-signature fields
// to zero. This leaves it in a form that may be compared with the
// signature to determine what instruction is present. This comparison
// requires only one u32 operation. The correct signature to use may
// be obtained by reading the opcode field.

pub fn mask_rtype(instr: u32) -> u32 {
    (mask!(7) << 25 | mask!(3) << 12) & instr
}

pub fn mask_isbtype(instr: u32) -> u32 {
    (mask!(3) << 12) & instr
}

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

#[derive(Debug, Clone, Copy)]
pub struct ExecFn32(pub fn(&mut Hart, instr: u32) -> Result<(), ExecutionError>);

/// This is a tree, containing a sequence of steps to decode an instruction
#[derive(Debug, Clone)]
pub enum SignatureDecoder {
    /// Variant for when another step of decoding is needed.
    Decoder {
        /// For the next decoding step, use this mask
        next_mask: u32,
        /// Then compare the value you get to this map to
        /// obtain the next decoding step
        value_map: HashMap<u32, SignatureDecoder>,
    },
    /// This is the leaf node, when the instruction is known
    Executer { xlen32_fn: Option<ExecFn32> },
}

impl SignatureDecoder {
    pub fn next_mask_and_map(&self) -> Option<(&u32, &HashMap<u32, SignatureDecoder>)> {
        match self {
            Self::Decoder {
                next_mask,
                value_map,
            } => Some((next_mask, value_map)),
            _ => None,
        }
    }

    pub fn decode(&self, instr: u32) -> Result<ExecFn32, DecodeError> {
        match self {
            Self::Decoder {
                next_mask,
                value_map,
            } => {
                let value = next_mask & instr;
                if let Some(next_decoder) = value_map.get(&value) {
                    next_decoder.decode(instr)
                } else {
                    Err(DecodeError::MissingValueInMap(instr))
                }
            }
            Self::Executer { xlen32_fn } => {
                if let Some(exec_fn) = xlen32_fn {
                    Ok(*exec_fn)
                } else {
                    Err(DecodeError::MissingExecFunction(instr))
                }
            }
        }
    }
}
