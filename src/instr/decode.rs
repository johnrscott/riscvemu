use super::fields::*;
use super::opcodes::*;
use std::fmt;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("got invalid or unimplemented opcode 0x{0:x}")]
    InvalidOpcode(u32),
}

/// RISC-V Instructions
///
/// Field names below correspond to the names in the
/// instruction set reference.
#[derive(Debug, Clone)]
pub enum Instr {
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
    /// set bit 0 to zero, and set pc = pc + result. The offset is 12
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
        mnemonic: String,
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
        mnemonic: String,
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
        mnemonic: String,
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
        mnemonic: String,
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
        mnemonic: String,
        dest: u8,
        src1: u8,
        src2: u8,
    },
}

/// Interpret the n least significant bits of
/// value (u32) as signed (i32) by manually
/// sign-extending based on bit n-1 and casting
/// to a signed type. When you use this macro,
/// make sure to include the type of the result
/// (e.g. x: i16 = interpret_as_signed!(...))
#[macro_export]
macro_rules! interpret_as_signed {
    ($value:expr, $n:expr) => {{
        let sign_bit = 1 & ($value >> ($n - 1));
        let sign_extended = if sign_bit == 1 {
            let all_ones = ((0 * $value).wrapping_sub(1));
            let sign_extension = all_ones - mask!($n);
            sign_extension | $value
        } else {
            $value
        };
        unsafe { std::mem::transmute(sign_extended) }
    }};
}
pub use interpret_as_signed;

impl Instr {
    pub fn from(instr: u32) -> Result<Self, DecodeError> {
        let op = opcode!(instr);
        match op {
            OP_LUI => {
                let dest = rd!(instr);
                let u_immediate = lui_u_immediate!(instr);
                Ok(Self::Lui { dest, u_immediate })
            }
            OP_AUIPC => {
                let dest = rd!(instr);
                let u_immediate = lui_u_immediate!(instr);
                Ok(Self::Auipc { dest, u_immediate })
            }
            OP_JAL => {
                let dest = rd!(instr);
                let offset = jal_offset!(instr);
                Ok(Self::Jal { dest, offset })
            }
            OP_JALR => {
                let dest = rd!(instr);
                let base = rs1!(instr);
                let offset = imm_itype!(instr);
                Ok(Self::Jalr { dest, base, offset })
            }
            OP_BRANCH => {
                let src1 = rs1!(instr);
                let src2 = rs2!(instr);
                let offset = imm_btype!(instr).try_into().unwrap();
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_BEQ => format!("beq"),
                    FUNCT3_BNE => format!("bne"),
                    FUNCT3_BLT => format!("blt"),
                    FUNCT3_BGE => format!("bge"),
                    FUNCT3_BLTU => format!("bltu"),
                    FUNCT3_BGEU => format!("bgeu"),
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
                let dest = rd!(instr);
                let base = rs1!(instr);
                let offset = imm_itype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_B => format!("lb"),
                    FUNCT3_H => format!("lh"),
                    FUNCT3_W => format!("lw"),
                    FUNCT3_BU => format!("lbu"),
                    FUNCT3_HU => format!("lhu"),
                    FUNCT3_WU => format!("lwu"),
                    FUNCT3_D => format!("ld"),
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
                let src = rs2!(instr);
                let base = rs1!(instr);
                let offset = imm_stype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_B => format!("sb"),
                    FUNCT3_H => format!("sh"),
                    FUNCT3_W => format!("sw"),
                    FUNCT3_D => format!("sd"),
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
                let src = rs1!(instr);
                let dest = rd!(instr);
                let mut i_immediate = imm_itype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_ADDI => format!("addi"),
                    FUNCT3_SLTI => format!("slti"),
                    FUNCT3_SLTIU => format!("sltiu"),
                    FUNCT3_ANDI => format!("andi"),
                    FUNCT3_ORI => format!("ori"),
                    FUNCT3_XORI => format!("xori"),
                    FUNCT3_SLLI => format!("slli"),
                    FUNCT3_SRLI => {
                        if is_arithmetic_shift!(instr) {
                            i_immediate = shamt!(instr).into();
                            format!("sra")
                        } else {
                            format!("srl")
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
            OP_IMM_32 => {
                let src = rs1!(instr);
                let dest = rd!(instr);
                let mut i_immediate = imm_itype!(instr);
                let funct3 = funct3!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_ADDI => format!("addiw"),
                    FUNCT3_SLLI => format!("slliw"),
                    FUNCT3_SRLI => {
                        if is_arithmetic_shift!(instr) {
                            i_immediate = shamt!(instr).into();
                            format!("sraw")
                        } else {
                            format!("srlw")
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
                let src1 = rs1!(instr);
                let src2 = rs2!(instr);
                let dest = rd!(instr);
                let funct3 = funct3!(instr);
                let funct7 = funct7!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_ADD => {
                        if funct7 == FUNCT7_SUB {
                            format!("sub")
                        } else {
                            format!("add")
                        }
                    }
                    FUNCT3_SLL => format!("sll"),
                    FUNCT3_SLT => format!("slt"),
                    FUNCT3_SLTU => format!("sltu"),
                    FUNCT3_XOR => format!("xor"),
                    FUNCT3_SRL => {
                        if is_arithmetic_shift!(instr) {
                            format!("sra")
                        } else {
                            format!("srl")
                        }
                    }
                    FUNCT3_OR => format!("or"),
                    FUNCT3_AND => format!("and"),
                    _ => panic!("Should change this to enum"),
                };
                Ok(Self::RegReg {
                    mnemonic,
                    dest,
                    src1,
                    src2,
                })
            }
            OP_32 => {
                let src1 = rs1!(instr);
                let src2 = rs2!(instr);
                let dest = rd!(instr);
                let funct3 = funct3!(instr);
                let funct7 = funct7!(instr);
                let mnemonic = match funct3 {
                    FUNCT3_ADD => {
                        if funct7 == FUNCT7_SUB {
                            format!("subw")
                        } else {
                            format!("addw")
                        }
                    }
                    FUNCT3_SLL => format!("sllw"),
                    FUNCT3_SRL => {
                        if funct7 == FUNCT7_SRA {
                            format!("sraw")
                        } else {
                            format!("srlw")
                        }
                    }
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

impl fmt::Display for Instr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            Self::Lui { dest, u_immediate } => {
                let u_immediate_signed: i32 = interpret_as_signed!(*u_immediate, 20);
                write!(f, "lui x{dest}, {u_immediate_signed}")
            }
            Self::Auipc { dest, u_immediate } => {
                let u_immediate_signed: i32 = interpret_as_signed!(*u_immediate, 20);
                write!(f, "auipc x{dest}, {u_immediate_signed}")
            }
            Self::Jal { dest, offset } => {
                let offset_signed: i32 = interpret_as_signed!(*offset, 21);
                write!(f, "jal x{dest}, {offset_signed}")
            }
            Self::Jalr { dest, base, offset } => {
                let offset_signed: i16 = interpret_as_signed!(*offset, 12);
                write!(f, "jalr x{dest}, x{base}, {offset_signed}")
            }
            Self::Branch {
                mnemonic,
                src1,
                src2,
                offset,
            } => {
                let offset_signed: i16 = interpret_as_signed!(*offset, 12);
                write!(f, "{mnemonic} x{src1}, x{src2}, {offset_signed}")
            }
            Self::Load {
                mnemonic,
                dest,
                base,
                offset,
            } => {
                let offset_signed: i16 = interpret_as_signed!(*offset, 12);
                write!(f, "{mnemonic} x{dest}, x{base}, {offset_signed}")
            }
            Self::Store {
                mnemonic,
                src,
                base,
                offset,
            } => {
                let offset_signed: i16 = interpret_as_signed!(*offset, 12);
                write!(f, "{mnemonic} x{src}, x{base}, {offset_signed}")
            }
            Self::RegImm {
                mnemonic,
                dest,
                src,
                i_immediate,
            } => {
                let i_immediate_signed: i16 = interpret_as_signed!(*i_immediate, 12);
                write!(f, "{mnemonic} x{dest}, x{src}, {i_immediate_signed}")
            }
            Self::RegReg {
                mnemonic,
                dest,
                src1,
                src2,
            } => {
                write!(f, "{mnemonic} x{dest}, x{src1}, x{src2}")
            }
        }
    }
}
