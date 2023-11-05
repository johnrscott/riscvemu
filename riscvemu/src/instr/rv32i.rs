//! RV32I base integer instruction set
//!
//! This file holds the instructions defined in chapter 2,
//! unprivileged specification version 20191213.
//!

use std::collections::HashMap;

use super::decode::decode_btype;
use super::decode::decode_itype;
use super::decode::decode_rtype;
use super::decode::decode_stype;
use super::decode::decode_utype;
use super::decode::isbtype_signature;
use super::decode::mask_isbtype;
use super::decode::Btype;
use super::decode::DecodeError;
use super::decode::Itype;
use super::decode::Ishtype;
use super::decode::Jtype;
use super::decode::Rtype;
use super::decode::SignatureDecoder;
use super::decode::Stype;
use super::decode::Utype;
use super::fields::*;
use super::opcodes::*;

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
    Beq(Btype),
    Bne(Btype),
    Blt(Btype),
    Bge(Btype),
    Bltu(Btype),
    Bgeu(Btype),
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
    Lb(Itype),
    Lh(Itype),
    Lw(Itype),
    Lbu(Itype),
    Lhu(Itype),
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
    Sb(Stype),
    Sh(Stype),
    Sw(Stype),
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
    Addi(Itype),
    Slti(Itype),
    Sltiu(Itype),
    Xori(Itype),
    Ori(Itype),
    Andi(Itype),
    Slli(Ishtype),
    Srli(Ishtype),
    Srai(Ishtype),
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
    Add(Rtype),
    Sub(Rtype),
    Sll(Rtype),
    Sltu(Rtype),
    Xor(Rtype),
    Srl(Rtype),
    Sra(Rtype),
    Or(Rtype),
    And(Rtype),
}
