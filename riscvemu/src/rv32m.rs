//! RV32M standard extension for integer multiplication and division
//!
//! This file holds the instructions defined in chapter 7,
//! unprivileged specification version 20191213.
//!

use crate::{
    decode::{Decoder, DecoderError},
    opcodes::{
        FUNCT3_DIV, FUNCT3_DIVU, FUNCT3_MUL, FUNCT3_MULH, FUNCT3_MULHSU,
        FUNCT3_MULHU, FUNCT3_REM, FUNCT3_REMU, FUNCT7_MULDIV, OP,
    },
    rv32i::{opcode_funct3_funct7_determined, Exec32},
};

use self::exec::{
    execute_div_rv32m, execute_divu_rv32m, execute_mul_rv32m,
    execute_mulh_rv32m, execute_mulhsu_rv32m, execute_mulhu_rv32m,
    execute_rem_rv32m, execute_remu_rv32m,
};

mod exec;

pub fn make_rv32m(decoder: &mut Decoder<Exec32>) -> Result<(), DecoderError> {
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MUL,
        FUNCT7_MULDIV,
        execute_mul_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULH,
        FUNCT7_MULDIV,
        execute_mulh_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULHSU,
        FUNCT7_MULDIV,
        execute_mulhsu_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULHU,
        FUNCT7_MULDIV,
        execute_mulhu_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_DIV,
        FUNCT7_MULDIV,
        execute_div_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_DIVU,
        FUNCT7_MULDIV,
        execute_divu_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_REM,
        FUNCT7_MULDIV,
        execute_rem_rv32m,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_REMU,
        FUNCT7_MULDIV,
        execute_remu_rv32m,
    )
}
