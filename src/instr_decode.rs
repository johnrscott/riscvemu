use std::fmt;
use crate::{instr_encode::*, instr_opcodes::*};

/// RISC-V Instructions
///
/// Field names below correspond to the names in the
/// instruction set reference.
#[derive(Debug)]
pub enum Instr {
    /// Load u_immediate into the high 20 bits of dest,
    /// filling the low 12 bits with zeros.
    Lui { dest: u8, u_immediate: u32 },
    /// Load imm into the high 20 bits of the pc
    Auipc { rd: u8, imm: u32 },
    /// Store the current pc+4 in rd, and set
    /// pc = pc + imm, where imm is a multiple of 2.
    Jal { rd: u8, imm: u32 },
    /// Store the current pc+4 in rd, and set
    /// pc = rs1 + imm (imm is a multiple of 2)
    Jalr { rd: u8, rs1: u8, imm: u32 },
    /// If rs1 == rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Beq { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 != rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Bne { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 < rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Blt { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 >= rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Bge { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 < rs2, set pc = pc + imm, where
    /// imm is a multiple of two, treating the
    /// contents of rs1 and rs2 as unsigned;
    /// else do nothing.
    Bltu { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 >= rs2, set pc = pc + imm, where
    /// imm is a multiple of two, treating the
    /// contents of rs1 and rs2 as unsigned;
    /// else do nothing.
    Bgeu { rs1: u8, rs2: u8, imm: u32 },
    /// Load the byte at address rs1 + imm into rd
    Lb { rd: u8, rs1: u8, imm: u32 },
    /// Load the halfword at address rs1 + imm into rd
    Lh { rd: u8, rs1: u8, imm: u32 },
    /// Load the word at address rs1 + imm into rd
    Lw { rd: u8, rs1: u8, imm: u32 },
    /// Store the byte in rs1 to address rs1 + imm
    Sb { rs1: u8, rs2: u8, imm: u32 },
    /// Store the halfword in rs1 to address rs1 + imm
    Sh { rs1: u8, rs2: u8, imm: u32 },
    /// Store the word in rs1 to address rs1 + imm
    Sw { rs1: u8, rs2: u8, imm: u32 },
}

macro_rules! opcode {
    ($instr:expr) => {
        extract_field!($instr, 6, 0)
    };
}

macro_rules! rd {
    ($instr:expr) => {{
	let rd: u8 = extract_field!($instr, 11, 7).try_into().unwrap();
	rd
    }};
}

macro_rules! lui_u_immediate {
    ($instr:expr) => {
        extract_field!($instr, 31, 12)
    };
}

impl Instr {
    pub fn from(instr: u32) -> Self {
        let op = opcode!(instr);
        match op {
            OP_LUI => {
                let dest = rd!(instr);
                let u_immediate = lui_u_immediate!(instr);
                Self::Lui { dest, u_immediate }
            },
	    _ => unimplemented!("Opcode 0b{op:b} is not yet implemented")
        }
    }
}

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
	match &self {
	    Self::Lui{dest, u_immediate} => write!(f, "lui x{dest}, {u_immediate}"),
	    _ => unimplemented!("Missing Display implementation for {:?}", &self),
	}
    }
}
