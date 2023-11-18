//! RV32I base integer instruction set
//!
//! This file holds the instructions defined in chapter 2,
//! unprivileged specification version 20191213.
//!

use crate::{
    decode::{Decoder, DecoderError, MaskWithValue},
    rv32i::exec::{
        execute_add_rv32i, execute_addi_rv32i, execute_and_rv32i, execute_andi_rv32i,
        execute_auipc_rv32i, execute_beq_rv32i, execute_bge_rv32i, execute_bgeu_rv32i,
        execute_blt_rv32i, execute_bltu_rv32i, execute_bne_rv32i, execute_jal_rv32i,
        execute_jalr_rv32i, execute_lb_rv32i, execute_lbu_rv32i, execute_lh_rv32i,
        execute_lhu_rv32i, execute_lui_rv32i, execute_lw_rv32i, execute_or_rv32i,
        execute_ori_rv32i, execute_sb_rv32i, execute_sh_rv32i, execute_sll_rv32i,
        execute_slli_rv32i, execute_slt_rv32i, execute_slti_rv32i, execute_sltiu_rv32i,
        execute_sltu_rv32i, execute_sra_rv32i, execute_srai_rv32i, execute_srl_rv32i,
        execute_srli_rv32i, execute_sub_rv32i, execute_sw_rv32i, execute_xor_rv32i,
        execute_xori_rv32i,
    },
    hart::{ExecutionError, Hart},
    mask,
    opcodes::{
        FUNCT3_ADD, FUNCT3_ADDI, FUNCT3_AND, FUNCT3_ANDI, FUNCT3_B, FUNCT3_BEQ, FUNCT3_BGE,
        FUNCT3_BGEU, FUNCT3_BLT, FUNCT3_BLTU, FUNCT3_BNE, FUNCT3_BU, FUNCT3_H, FUNCT3_HU,
        FUNCT3_JALR, FUNCT3_OR, FUNCT3_ORI, FUNCT3_SLL, FUNCT3_SLLI, FUNCT3_SLT, FUNCT3_SLTI,
        FUNCT3_SLTIU, FUNCT3_SLTU, FUNCT3_SRA, FUNCT3_SRAI, FUNCT3_SRL, FUNCT3_SRLI, FUNCT3_SUB,
        FUNCT3_W, FUNCT3_XOR, FUNCT3_XORI, FUNCT7_ADD, FUNCT7_AND, FUNCT7_OR, FUNCT7_SLL,
        FUNCT7_SLLI, FUNCT7_SLT, FUNCT7_SLTU, FUNCT7_SRA, FUNCT7_SRAI, FUNCT7_SRL, FUNCT7_SRLI,
        FUNCT7_SUB, FUNCT7_XOR, OP, OP_AUIPC, OP_BRANCH, OP_IMM, OP_JAL, OP_JALR, OP_LOAD, OP_LUI,
        OP_STORE,
    },
};

mod exec;

pub type Exec32 = fn(&mut Hart, u32) -> Result<(), ExecutionError>;

/// In RV32I and RV64I, If branch is taken, set pc = pc + offset,
/// where offset is a multiple of two; else do nothing. The
/// offset is 13 bits long.
///
/// The condition for branch taken depends on the value in
/// mnemonic, which is one of:
/// - "beq": src1 == src2
/// - "bne": src1 != src2
/// - "blt": src1 < src2 as signed integers
/// - "bge": src1 >= src2 as signed integers
/// - "bltu": src1 < src2 as unsigned integers
/// - "bgeu": src1 >= src2 as unsigned integers
///
/// Only on branch-taken, an instruction-address-misaligned
/// exception is generated if the target pc is not 4-byte
/// aligned.
pub enum Branch {
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
}

/// In RV32I and RV64I, load the data at address base + offset
/// into dest. The offset is 12 bits long.
///
/// The size of data, and the way it is loaded into dest, depends
/// on the mnemonic, as follows:
///
/// In RV32I:
/// - "lb": load a byte, sign extend in dest
/// - "lh": load a halfword, sign extend in dest
/// - "lw": load a word
/// - "lbu": load a byte, zero extend in dest
/// - "lhu": load a halfword, zero extend in dest
///
/// In RV64I:
/// - "lw": load a word, sign extend in dest
/// - "lwu": load a word, zero extend in dest
/// - "ld": load a doubleword
///
/// Loads do not need to be aligned
pub enum Load {
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
}

/// In RV32I and RV64I, load the data at src into address base +
/// offset. The offset is 12 bits long.
///
/// The mnemonic determines the width of data that is stored to
/// memory:
///
/// In RV32I:
/// - "sb": store a byte
/// - "sh": store a halfword
/// - "sw": store a word
///
/// In RV64I:
/// - "sd": store a doubleword
///
/// Stores do not need to be aligned
pub enum Store {
    Sb,
    Sh,
    Sw,
}

/// In RV32I and RV64I, perform an operation between the value in
/// register src and the sign-extended version of the 12-bit
/// i_immediate.
///
/// The operation performed is determined by the mnemonic as follows:
/// - "addi": dest = src + i_immediate
/// - "slti": dest = (src < i_immediate) ? 1 : 0, signed comparison
/// - "sltiu": dest = (src < i_immediate) ? 1 : 0, unsigned comparison
/// - "andi": dest = src & i_immediate
/// - "ori": dest = src | i_immediate
/// - "xori": dest = src ^ i_immediate
/// - "slli": dest = src << (0x1f & i_immediate)
/// - "srli": dest = src >> (0x1f & i_immediate) (logical)
/// - "srai": dest = src >> (0x1f & i_immediate) (arithmetic)
///
/// In RV64I, the shift operators
///
pub enum RegImm {
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
}

/// In RV32I and RV64I, perform an operation between the values in
/// src1 and src2 and place the result in dest
///
/// In RV32I, the operation performed is determined by the mnemonic
/// as follows:
/// - "add": dest = src1 + src2
/// - "sub": dest = src1 - src2
/// - "slt": dest = (src1 < src2) ? 1 : 0, signed comparison
/// - "sltu": dest = (src1 < src2) ? 1 : 0, unsigned comparison
/// - "and": dest = src1 & src2
/// - "or": dest = src1 | src2
/// - "xor": dest = src1 ^ src2
/// - "sll": dest = src1 << (0x1f & src2)
/// - "srl": dest = src1 >> (0x1f & src2) (logical)
/// - "sra": dest = src1 >> (0x1f & src2) (arithmetic)
///
/// In RV64I, the shift operators using the bottom 6 bits of
/// src2 as the shift amount: (0x3f & src2). In addition, the
/// following instructions operate on the low 32 bits of the
/// registers:
/// - "addw"
/// - "subw"
/// - "sllw"
/// - "srlw"
/// - "sraw"
///
pub enum RegReg {
    Add,
    Sub,
    Sll,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
}

/*
/// RISC-V Instructions
///
/// Field names below correspond to the names in the
/// instruction set reference.
#[derive(Debug, Clone)]
pub enum Rv32i {
    /// In RV32I and RV64I, load u_immediate into dest[31:12] bits of
    /// dest, filling the low 12 bits with zeros. In RV64I, also sign
    /// extend the result to the high bits of dest. u_immediate is 20
    /// bits long.
    Lui(Utype),
    /// In RV32I, concatenate u_immediate with 12 low-order zeros, add
    /// pc to the the result, and place the result in dest. In RV64I,
    /// sign extend the result before adding to the pc. u_immediate is
    /// 20 bits long.
    Auipc(Utype),
    /// In RV32I and RV64I, store pc+4 in dest, and set pc = pc +
    /// offset, where offset is a multiple of 2. Offset is 21 bits
    /// long. An instruction-address-misaligned exception is generated
    /// if the target pc is not 4-byte aligned.
    Jal(Jtype),
    /// In RV32I and RV64I, store pc+4 in dest, compute base + offset,
    /// set bit 0 to zero, and set pc = result. The offset is 12
    /// bits long (and may be even or odd). An
    /// instruction-address-misaligned exception is generated if the
    /// target pc is not 4-byte aligned.
    Jalr(Itype),
}
*/

pub fn opcode_determined(
    decoder: &mut Decoder<Exec32>,
    opcode: u32,
    exec: Exec32,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![MaskWithValue {
        mask: mask!(7),
        value: opcode,
    }];
    decoder.push_instruction(masks_with_values, exec)
}

pub fn opcode_funct3_determined(
    decoder: &mut Decoder<Exec32>,
    opcode: u32,
    funct3: u32,
    exec: Exec32,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![
        MaskWithValue {
            mask: (mask!(3)) << 12,
            value: funct3 << 12,
        },
        MaskWithValue {
            mask: mask!(7),
            value: opcode,
        },
    ];
    decoder.push_instruction(masks_with_values, exec)
}

/// This also covers the shift instructions which use a special version
/// if I-type.
pub fn opcode_funct3_funct7_determined(
    decoder: &mut Decoder<Exec32>,
    opcode: u32,
    funct3: u32,
    funct7: u32,
    exec: Exec32,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![
	// funct3/funct7 combined into one step -- might be OK,
	// the decoder will complain if it is ambiguous
        MaskWithValue {
            mask: ((mask!(7)) << 25) | ((mask!(3)) << 12),
            value: (funct7 << 25) | (funct3 << 12),
        },
        MaskWithValue {
            mask: mask!(7),
            value: opcode,
        },
    ];
    decoder.push_instruction(masks_with_values, exec)
}

pub fn make_rv32i(decoder: &mut Decoder<Exec32>) -> Result<(), DecoderError> {
    // Opcode determines instruction
    opcode_determined(decoder, OP_LUI, execute_lui_rv32i)?;
    opcode_determined(decoder, OP_AUIPC, execute_auipc_rv32i)?;
    opcode_determined(decoder, OP_JAL, execute_jal_rv32i)?;

    // Opcode and funct3 determines instruction
    opcode_funct3_determined(decoder, OP_JALR, FUNCT3_JALR, execute_jalr_rv32i)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BEQ, execute_beq_rv32i)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BNE, execute_bne_rv32i)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BLT, execute_blt_rv32i)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BGE, execute_bge_rv32i)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BLTU, execute_bltu_rv32i)?;
    opcode_funct3_determined(decoder, OP_BRANCH, FUNCT3_BGEU, execute_bgeu_rv32i)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_B, execute_lb_rv32i)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_H, execute_lh_rv32i)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_W, execute_lw_rv32i)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_BU, execute_lbu_rv32i)?;
    opcode_funct3_determined(decoder, OP_LOAD, FUNCT3_HU, execute_lhu_rv32i)?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_B, execute_sb_rv32i)?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_H, execute_sh_rv32i)?;
    opcode_funct3_determined(decoder, OP_STORE, FUNCT3_W, execute_sw_rv32i)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ADDI, execute_addi_rv32i)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_SLTI, execute_slti_rv32i)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_SLTIU, execute_sltiu_rv32i)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_XORI, execute_xori_rv32i)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ORI, execute_ori_rv32i)?;
    opcode_funct3_determined(decoder, OP_IMM, FUNCT3_ANDI, execute_andi_rv32i)?;

    // Shift instructions (opcode, funct3, and part of immediate determined)
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SLLI,
        FUNCT7_SLLI,
        execute_slli_rv32i,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SRLI,
        FUNCT7_SRLI,
        execute_srli_rv32i,
    )?;
    opcode_funct3_funct7_determined(
        decoder,
        OP_IMM,
        FUNCT3_SRAI,
        FUNCT7_SRAI,
        execute_srai_rv32i,
    )?;

    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_ADD, FUNCT7_ADD, execute_add_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_SUB, FUNCT7_SUB, execute_sub_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_SLL, FUNCT7_SLL, execute_sll_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_SLT, FUNCT7_SLT, execute_slt_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_SLTU, FUNCT7_SLTU, execute_sltu_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_XOR, FUNCT7_XOR, execute_xor_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_SRL, FUNCT7_SRL, execute_srl_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_SRA, FUNCT7_SRA, execute_sra_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_OR, FUNCT7_OR, execute_or_rv32i)?;
    opcode_funct3_funct7_determined(decoder, OP, FUNCT3_AND, FUNCT7_AND, execute_and_rv32i)
}
