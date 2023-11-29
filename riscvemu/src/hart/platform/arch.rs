use super::{eei::Eei, rv32i::*, rv32m::*, rv32zicsr::*, Instr};
use crate::{
    decode::{Decoder, DecoderError, MaskWithValue},
    opcodes::{
        FUNCT3_ADD, FUNCT3_ADDI, FUNCT3_AND, FUNCT3_ANDI, FUNCT3_B, FUNCT3_BEQ,
        FUNCT3_BGE, FUNCT3_BGEU, FUNCT3_BLT, FUNCT3_BLTU, FUNCT3_BNE,
        FUNCT3_BU, FUNCT3_CSRRC, FUNCT3_CSRRCI, FUNCT3_CSRRS, FUNCT3_CSRRSI,
        FUNCT3_CSRRW, FUNCT3_CSRRWI, FUNCT3_DIV, FUNCT3_DIVU, FUNCT3_H,
        FUNCT3_HU, FUNCT3_JALR, FUNCT3_MUL, FUNCT3_MULH, FUNCT3_MULHSU,
        FUNCT3_MULHU, FUNCT3_OR, FUNCT3_ORI, FUNCT3_REM, FUNCT3_REMU,
        FUNCT3_SLL, FUNCT3_SLLI, FUNCT3_SLT, FUNCT3_SLTI, FUNCT3_SLTIU,
        FUNCT3_SLTU, FUNCT3_SRA, FUNCT3_SRAI, FUNCT3_SRL, FUNCT3_SRLI,
        FUNCT3_SUB, FUNCT3_W, FUNCT3_XOR, FUNCT3_XORI, FUNCT7_ADD, FUNCT7_AND,
        FUNCT7_MULDIV, FUNCT7_OR, FUNCT7_SLL, FUNCT7_SLLI, FUNCT7_SLT,
        FUNCT7_SLTU, FUNCT7_SRA, FUNCT7_SRAI, FUNCT7_SRL, FUNCT7_SRLI,
        FUNCT7_SUB, FUNCT7_XOR, OP, OP_AUIPC, OP_BRANCH, OP_IMM, OP_JAL,
        OP_JALR, OP_LOAD, OP_LUI, OP_STORE, OP_SYSTEM,
    },
    utils::mask,
};

/// The intention of this kind of function (generic on EEI) is to provide
/// a way to separate the decoding of the instruction from the actual
/// implementation of the execution environment
pub fn opcode_determined<E: Eei>(
    decoder: &mut Decoder<Instr<E>>,
    opcode: u32,
    instr: Instr<E>,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![MaskWithValue {
        mask: mask(7),
        value: opcode,
    }];
    decoder.push_instruction(masks_with_values, instr)
}

/// See comment for opcode_determined
pub fn opcode_funct3_determined<E: Eei>(
    decoder: &mut Decoder<Instr<E>>,
    opcode: u32,
    funct3: u32,
    instr: Instr<E>,
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
    decoder.push_instruction(masks_with_values, instr)
}

/// This also covers the shift instructions which use a special version
/// if I-type.
pub fn opcode_funct3_funct7_determined<E: Eei>(
    decoder: &mut Decoder<Instr<E>>,
    opcode: u32,
    funct3: u32,
    funct7: u32,
    instr: Instr<E>,
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
    decoder.push_instruction(masks_with_values, instr)
}

pub fn make_rv32i<E: Eei>(
    decoder: &mut Decoder<Instr<E>>,
) -> Result<(), DecoderError> {
    // Opcode determines instruction
    opcode_determined(decoder, OP_LUI, lui())?;
    opcode_determined(decoder, OP_AUIPC, auipc())?;
    opcode_determined(decoder, OP_JAL, jal())?;

    // Opcode and funct3 determines instruction
    opcode_funct3_determined(decoder, OP_JALR, FUNCT3_JALR, jalr())?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BEQ, beq())?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BNE, bne())?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BLT, blt())?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BGE, bge())?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BLTU, bltu())?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BGEU, bgeu())?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_B, lb())?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_H, lh())?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_W, lw())?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_BU, lbu())?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_HU, lhu())?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_B, sb())?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_H, sh())?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_W, sw())?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ADDI, addi())?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_SLTI, slti())?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_SLTIU, sltiu())?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_XORI, xori())?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ORI, ori())?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ANDI, andi())?;

    // Shift instructions (opcode, funct3, and part of immediate determined)
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SLLI,
        FUNCT7_SLLI,
        slli(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SRLI,
        FUNCT7_SRLI,
        srli(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SRAI,
        FUNCT7_SRAI,
        srai(),
    )?;

    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_ADD,
        FUNCT7_ADD,
        add(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SUB,
        FUNCT7_SUB,
        sub(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SLL,
        FUNCT7_SLL,
        sll(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SLT,
        FUNCT7_SLT,
        slt(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SLTU,
        FUNCT7_SLTU,
        sltu(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_XOR,
        FUNCT7_XOR,
        xor(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SRL,
        FUNCT7_SRL,
        srl(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_SRA,
        FUNCT7_SRA,
        sra(),
    )?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_OR, FUNCT7_OR, or())?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_AND, FUNCT7_AND, and())
}

pub fn make_rv32m<E: Eei>(
    decoder: &mut Decoder<Instr<E>>,
) -> Result<(), DecoderError> {
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MUL,
        FUNCT7_MULDIV,
        mul(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULH,
        FUNCT7_MULDIV,
        mulh(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULHSU,
        FUNCT7_MULDIV,
        mulhsu(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_MULHU,
        FUNCT7_MULDIV,
        mulhu(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_DIV,
        FUNCT7_MULDIV,
        div(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_DIVU,
        FUNCT7_MULDIV,
        divu(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_REM,
        FUNCT7_MULDIV,
        rem(),
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP,
        FUNCT3_REMU,
        FUNCT7_MULDIV,
        remu(),
    )
}

pub fn make_rv32zicsr<E: Eei>(
    decoder: &mut Decoder<Instr<E>>,
) -> Result<(), DecoderError> {
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRW, csrrw())?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRS, csrrs())?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRC, csrrc())?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRWI, csrrwi())?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRSI, csrrsi())?;
    opcode_funct3_determined(decoder, OP_SYSTEM, FUNCT3_CSRRCI, csrrci())
}
