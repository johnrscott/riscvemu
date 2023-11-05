//! RV32I base integer instruction set
//!
//! This file holds the instructions defined in chapter 2,
//! unprivileged specification version 20191213. 
//! 

use super::decode::DecodeError;
use super::fields::*;
use super::opcodes::*;

pub enum Multiply {
    Mul,
    Mulh,
    Mulhsu,
    Mulhu,
}

pub enum DivRem {
    Div,
    Divu,
    Rem,
    Remu,
}

#[derive(Debug, Clone)]
pub enum Rv32m {
    Multiply {
	Mnemonic: Multiply,
	dest: u8,
	multiplicand: u8,
	multiplier: u8,
    },
    DivRem {
	Mnemonic: DivRem,
	dest: u8,
	divisor: u8,
	dividend: u8,
    }    
}

#[derive(Debug, Copy, Clone)]
pub enum Branch {
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
}

#[derive(Debug, Copy, Clone)]
pub enum Load {
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
}

#[derive(Debug, Copy, Clone)]
pub enum Store {
    Sb,
    Sh,
    Sw,
}

#[derive(Debug, Copy, Clone)]
pub enum RegImm {
    Addi,
    Slti,
    Sltiu,
    Andi,
    Ori,
    Xori,
    Slli,
    Srli,
    Srai,
}

#[derive(Debug, Copy, Clone)]
pub enum RegReg {
    Add,
    Sub,
    Slt,
    Sltu,
    And,
    Or,
    Xor,
    Sll,
    Srl,
    Sra,
}

impl Rv32m {
    pub fn from(instr: u32) -> Result<Self, DecodeError> {
        let op = opcode!(instr);
        match op {
	    OP => {
		let rs1 = rs1!(instr);
		let rs2 = rs2!(instr);
		let dest = rd!(instr);
		let funct3 = funct3!(instr);
		let funct7 = funct7!(instr);
		let mnemonic = match funct3 {
		    FUNCT3_MUL => Multiply::Mul,
		    FUNCT3_MULH => Multiply::Mulh,
		    FUNCT3_MULHSU => Multiply::Mulhsu,
		    FUNCT3_MULHU => Multiply::Mulhu,
		    FUNCT3_MULH => Multiply::Mulh,
		    
		    _ => panic!("Should change this to enum"),
		};
		Ok(Self::RegReg {
		    mnemonic,
		    dest,
		    src1,
		    src2,
		})
	    }
	    _ => Err(DecodeError::InvalidOpcode(op)),
	}
    }
}
