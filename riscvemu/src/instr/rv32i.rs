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
use super::decode::DecodeError;
use super::decode::Itype;
use super::decode::Rtype;
use super::decode::SBtype;
use super::decode::UJtype;
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
    Lui { dest: u8, u_immediate: u32 },
    /// In RV32I, concatenate u_immediate with 12 low-order zeros, add
    /// pc to the the result, and place the result in dest. In RV64I,
    /// sign extend the result before adding to the pc. u_immediate is
    /// 20 bits long.
    Auipc { dest: u8, u_immediate: u32 },
    /// In RV32I and RV64I, store pc+4 in dest, and set pc = pc +
    /// offset, where offset is a multiple of 2. Offset is 21 bits
    /// long. An instruction-address-misaligned exception is generated
    /// if the target pc is not 4-byte aligned.
    Jal { dest: u8, offset: u32 },
    /// In RV32I and RV64I, store pc+4 in dest, compute base + offset,
    /// set bit 0 to zero, and set pc = result. The offset is 12
    /// bits long (and may be even or odd). An
    /// instruction-address-misaligned exception is generated if the
    /// target pc is not 4-byte aligned.
    Jalr { dest: u8, base: u8, offset: u16 },
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
    Branch {
        mnemonic: Branch,
        src1: u8,
        src2: u8,
        offset: u16,
    },
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
    Load {
        mnemonic: Load,
        dest: u8,
        base: u8,
        offset: u16,
    },
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
    Store {
        mnemonic: Store,
        src: u8,
        base: u8,
        offset: u16,
    },
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
    RegImm {
        mnemonic: RegImm,
        dest: u8,
        src: u8,
        i_immediate: u16,
    },
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
    RegReg {
        mnemonic: RegReg,
        dest: u8,
        src1: u8,
        src2: u8,
    },
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

/// lui is completely determined by the opcode
fn decode_lui(instr: u32) -> Rv32i {
    let UJtype { rd, imm } = decode_utype(instr);
    Rv32i::Lui {
        dest: rd,
        u_immediate: imm,
    }
}

fn decode_beq(instr: u32) -> Rv32i {
    let SBtype { rs1, rs2, imm } = decode_btype(instr);
    Rv32i::Branch {
        mnemonic: Branch::Beq,
        src1: rs1,
        src2: rs2,
        offset: imm,
    }
}

fn decode_lb(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::Load {
        mnemonic: Load::Lb,
        dest: rd,
        base: rs1,
        offset: imm,
    }
}

fn decode_sb(instr: u32) -> Rv32i {
    let SBtype { rs1, rs2, imm } = decode_stype(instr);
    Rv32i::Store {
        mnemonic: Store::Sb,
        src: rs2,
        base: rs1,
        offset: imm,
    }
}

fn decode_addi(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Addi,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_slli(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Slli,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_slti(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Slti,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_sltiu(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Sltiu,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_xori(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Xori,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_ori(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Ori,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_andi(instr: u32) -> Rv32i {
    let Itype { rs1, imm, rd } = decode_itype(instr);
    Rv32i::RegImm {
        mnemonic: RegImm::Andi,
        dest: rd,
        src: rs1,
        i_immediate: imm,
    }
}

fn decode_add(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Add,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_sub(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Sub,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_sll(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Sll,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_slt(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Slt,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_sltu(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Sltu,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_xor(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Xor,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_or(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::Or,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

fn decode_and(instr: u32) -> Rv32i {
    let Rtype { rs1, rs2, rd } = decode_rtype(instr);
    Rv32i::RegReg {
        mnemonic: RegReg::And,
        dest: rd,
        src1: rs1,
        src2: rs2,
    }
}

pub fn decoders() -> HashMap<u32, fn(i32)->Rv32i> {
    let mut signature_map = HashMap::new();
    signature_map.insert(12, decode_lui)
}

impl Rv32i {
    pub fn from(instr: u32) -> Result<Self, DecodeError> {
        let op = opcode!(instr);
        match op {
            OP_LUI => Ok(decode_lui(instr)),
            OP_AUIPC => {
                // auipc is completely determined by the opcode
                let dest = rd!(instr);
                let u_immediate = lui_u_immediate!(instr);
                Ok(Self::Auipc { dest, u_immediate })
            }
            OP_JAL => {
                // jal is completely determined by the opcode
                let dest = rd!(instr);
                let offset = jal_offset!(instr);
                Ok(Self::Jal { dest, offset })
            }
            OP_JALR => {
                // jalr is completely determined by the opcode
                let dest = rd!(instr);
                let base = rs1!(instr);
                let offset = imm_itype!(instr);
                Ok(Self::Jalr { dest, base, offset })
            }
            OP_BRANCH => {
                // Conditional branches are decoded by seeing a BRANCH
                // opcode, and then using funct3 to determine which
                // branch instruction is present
                //
                let src1 = rs1!(instr);
                let src2 = rs2!(instr);
                let offset = imm_btype!(instr).try_into().unwrap();
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_BEQ => Branch::Beq,
                    FUNCT3_BNE => Branch::Bne,
                    FUNCT3_BLT => Branch::Blt,
                    FUNCT3_BGE => Branch::Bge,
                    FUNCT3_BLTU => Branch::Bltu,
                    FUNCT3_BGEU => Branch::Bgeu,
                    _ => panic!("Should change this to enum"),
                };
                Ok(Self::Branch {
                    mnemonic,
                    src1,
                    src2,
                    offset,
                })
            }
            OP_LOAD => {
                // Loads are decoded by seeing a LOAD opcode, and then
                // using funct3 to determine which load instruction is
                // present.
                let dest = rd!(instr);
                let base = rs1!(instr);
                let offset = imm_itype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_B => Load::Lb,
                    FUNCT3_H => Load::Lh,
                    FUNCT3_W => Load::Lw,
                    FUNCT3_BU => Load::Lbu,
                    FUNCT3_HU => Load::Lhu,
                    _ => panic!("Should change this to enum"),
                };
                Ok(Self::Load {
                    mnemonic,
                    dest,
                    base,
                    offset,
                })
            }
            OP_STORE => {
                // Stores are decoded by seeing a STORE opcode, and
                // then using funct3 to determine which store
                // instruction is present.
                let src = rs2!(instr);
                let base = rs1!(instr);
                let offset = imm_stype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_B => Store::Sb,
                    FUNCT3_H => Store::Sh,
                    FUNCT3_W => Store::Sw,
                    _ => panic!("Should change this to enum"),
                };
                Ok(Self::Store {
                    mnemonic,
                    src,
                    base,
                    offset,
                })
            }
            OP_IMM => {
                // Register-immediate computational instruction are
                // decoded by seeing an IMM opcode, using
                // funct3 to determine which instruction is present,
                // and then if funct3 is srli, using bit 30 to distinguish
                // between srli and srai.
                let src = rs1!(instr);
                let dest = rd!(instr);
                let mut i_immediate = imm_itype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_ADDI => RegImm::Addi,
                    FUNCT3_SLTI => RegImm::Slti,
                    FUNCT3_SLTIU => RegImm::Sltiu,
                    FUNCT3_ANDI => RegImm::Andi,
                    FUNCT3_ORI => RegImm::Ori,
                    FUNCT3_XORI => RegImm::Xori,
                    FUNCT3_SLLI => RegImm::Slli,
                    FUNCT3_SRLI => {
                        if is_arithmetic_shift!(instr) {
                            i_immediate = shamt!(instr).into();
                            RegImm::Srai
                        } else {
                            RegImm::Srli
                        }
                    }
                    _ => panic!("Should change this to enum"),
                };
                Ok(Self::RegImm {
                    mnemonic,
                    dest,
                    src,
                    i_immediate,
                })
            }
            OP => {
                // Register-register computational instruction are
                // decoded by seeing an OP opcode, using
                // funct3 to determine which instruction is present,
                // and then
                // - if funct3 is srl, using bit 30 to distinguish
                //   between srl and sra.
                // - if funct3 is add, using bit 30 to distinguish
                //   between add and sub
                let src1 = rs1!(instr);
                let src2 = rs2!(instr);
                let dest = rd!(instr);
                let funct3 = funct3!(instr);
                let funct7 = funct7!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_ADD => {
                        if funct7 == FUNCT7_SUB {
                            RegReg::Sub
                        } else {
                            RegReg::Add
                        }
                    }
                    FUNCT3_SLL => RegReg::Sll,
                    FUNCT3_SLT => RegReg::Slt,
                    FUNCT3_SLTU => RegReg::Sltu,
                    FUNCT3_XOR => RegReg::Xor,
                    FUNCT3_SRL => {
                        if is_arithmetic_shift!(instr) {
                            RegReg::Sra
                        } else {
                            RegReg::Srl
                        }
                    }
                    FUNCT3_OR => RegReg::Or,
                    FUNCT3_AND => RegReg::And,
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
