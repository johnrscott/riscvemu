use super::{
    eei::Eei,
    rv32i::{
        execute_add, execute_addi, execute_and, execute_andi, execute_auipc,
        execute_beq, execute_bge, execute_bgeu, execute_blt, execute_bltu,
        execute_bne, execute_jal, execute_jalr, execute_lb, execute_lbu,
        execute_lh, execute_lhu, execute_lui, execute_lw, execute_or,
        execute_ori, execute_sb, execute_sh, execute_sll, execute_slli,
        execute_slt, execute_slti, execute_sltiu, execute_sltu, execute_sra,
        execute_srai, execute_srl, execute_srli, execute_sub, execute_sw,
        execute_xor, execute_xori,
    },
    rv32m::{
        execute_div, execute_divu, execute_mul, execute_mulh, execute_mulhsu,
        execute_mulhu, execute_rem, execute_remu,
    },
    rv32zicsr::{execute_csrrs, execute_csrrw, execute_csrrc},
    ExecuteInstr,
};
use crate::{
    decode::{Decoder, DecoderError, MaskWithValue},
    opcodes::{
        FUNCT3_ADD, FUNCT3_ADDI, FUNCT3_AND, FUNCT3_ANDI, FUNCT3_B, FUNCT3_BEQ,
        FUNCT3_BGE, FUNCT3_BGEU, FUNCT3_BLT, FUNCT3_BLTU, FUNCT3_BNE,
        FUNCT3_BU, FUNCT3_CSRRS, FUNCT3_CSRRW, FUNCT3_DIV, FUNCT3_DIVU,
        FUNCT3_H, FUNCT3_HU, FUNCT3_JALR, FUNCT3_MUL, FUNCT3_MULH,
        FUNCT3_MULHSU, FUNCT3_MULHU, FUNCT3_OR, FUNCT3_ORI, FUNCT3_REM,
        FUNCT3_REMU, FUNCT3_SLL, FUNCT3_SLLI, FUNCT3_SLT, FUNCT3_SLTI,
        FUNCT3_SLTIU, FUNCT3_SLTU, FUNCT3_SRA, FUNCT3_SRAI, FUNCT3_SRL,
        FUNCT3_SRLI, FUNCT3_SUB, FUNCT3_W, FUNCT3_XOR, FUNCT3_XORI, FUNCT7_ADD,
        FUNCT7_AND, FUNCT7_MULDIV, FUNCT7_OR, FUNCT7_SLL, FUNCT7_SLLI,
        FUNCT7_SLT, FUNCT7_SLTU, FUNCT7_SRA, FUNCT7_SRAI, FUNCT7_SRL,
        FUNCT7_SRLI, FUNCT7_SUB, FUNCT7_XOR, OP, OP_AUIPC, OP_BRANCH, OP_IMM,
        OP_JAL, OP_JALR, OP_LOAD, OP_LUI, OP_STORE, OP_SYSTEM, FUNCT3_CSRRC,
    },
    utils::mask,
};

/// The intention of this kind of function (generic on EEI) is to provide
/// a way to separate the decoding of the instruction from the actual
/// implementation of the execution environment
pub fn opcode_determined<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
    opcode: u32,
    exec: ExecuteInstr<E>,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![MaskWithValue {
        mask: mask(7),
        value: opcode,
    }];
    decoder.push_instruction(masks_with_values, exec)
}

/// See comment for opcode_determined
pub fn opcode_funct3_determined<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
    opcode: u32,
    funct3: u32,
    exec: ExecuteInstr<E>,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![
        MaskWithValue {
            mask: (mask(3)) << 12,
            value: funct3 << 12,
        },
        MaskWithValue {
            mask: mask(7),
            value: opcode,
        },
    ];
    decoder.push_instruction(masks_with_values, exec)
}

/// This also covers the shift instructions which use a special version
/// if I-type.
pub fn opcode_funct3_funct7_determined<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
    opcode: u32,
    funct3: u32,
    funct7: u32,
    exec: ExecuteInstr<E>,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![
        MaskWithValue {
            mask: (mask(7)) << 25,
            value: funct7 << 25,
        },
        MaskWithValue {
            mask: (mask(3)) << 12,
            value: funct3 << 12,
        },
        MaskWithValue {
            mask: mask(7),
            value: opcode,
        },
    ];
    decoder.push_instruction(masks_with_values, exec)
}

pub fn make_rv32i<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
) -> Result<(), DecoderError> {
    // Opcode determines instruction
    opcode_determined(decoder, OP_LUI, execute_lui)?;
    opcode_determined(decoder, OP_AUIPC, execute_auipc)?;
    opcode_determined(decoder, OP_JAL, execute_jal)?;

    // Opcode and funct3 determines instruction
    opcode_funct3_determined(decoder, OP_JALR, FUNCT3_JALR, execute_jalr)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BEQ, execute_beq)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BNE, execute_bne)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BLT, execute_blt)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BGE, execute_bge)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BLTU, execute_bltu)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BGEU, execute_bgeu)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_B, execute_lb)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_H, execute_lh)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_W, execute_lw)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_BU, execute_lbu)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_HU, execute_lhu)?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_B, execute_sb)?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_H, execute_sh)?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_W, execute_sw)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ADDI, execute_addi)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_SLTI, execute_slti)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_SLTIU, execute_sltiu)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_XORI, execute_xori)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ORI, execute_ori)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ANDI, execute_andi)?;

    // Shift instructions (opcode, funct3, and part of immediate determined)
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SLLI,
        FUNCT7_SLLI,
        execute_slli,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SRLI,
        FUNCT7_SRLI,
        execute_srli,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SRAI,
        FUNCT7_SRAI,
        execute_srai,
    )?;

    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_ADD,
        FUNCT7_ADD,
        execute_add,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SUB,
        FUNCT7_SUB,
        execute_sub,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SLL,
        FUNCT7_SLL,
        execute_sll,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SLT,
        FUNCT7_SLT,
        execute_slt,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SLTU,
        FUNCT7_SLTU,
        execute_sltu,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_XOR,
        FUNCT7_XOR,
        execute_xor,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SRL,
        FUNCT7_SRL,
        execute_srl,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SRA,
        FUNCT7_SRA,
        execute_sra,
    )?;
    opcode_funct3_funct7_determined(
        decoder, OP, FUNCT3_OR, FUNCT7_OR, execute_or,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_AND,
        FUNCT7_AND,
        execute_and,
    )
}

pub fn make_rv32m<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
) -> Result<(), DecoderError> {
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MUL,
        FUNCT7_MULDIV,
        execute_mul,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULH,
        FUNCT7_MULDIV,
        execute_mulh,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULHSU,
        FUNCT7_MULDIV,
        execute_mulhsu,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULHU,
        FUNCT7_MULDIV,
        execute_mulhu,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_DIV,
        FUNCT7_MULDIV,
        execute_div,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_DIVU,
        FUNCT7_MULDIV,
        execute_divu,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_REM,
        FUNCT7_MULDIV,
        execute_rem,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_REMU,
        FUNCT7_MULDIV,
        execute_remu,
    )
}

pub fn make_rv32zicsr<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
) -> Result<(), DecoderError> {
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRW, execute_csrrw)?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRS, execute_csrrs)?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRC, execute_csrrc)
}
