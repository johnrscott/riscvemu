#[derive(Clone, Copy)]
pub struct Rtype {
    pub rs1: u8,
    pub rs2: u8,
    pub rd: u8,
}

impl Rtype {
    /// get the signature from the instruction to 
    /// compare with map key. Signature does not include the opcode.
pub fn mask(instr: u32) -> u32 {
    (mask!(7) << 25 | mask!(3) << 12 | mask!(7)) & instr
}

    /// build signature that uniquely IDs the instruction
    pub fn signature(funct3: u32, funct7: u32) -> u32 {
/// make the signature here (no opcode)
    }
}

/// map opcodes to a type that stores
/// - Instruction type (e.g. Rtype)
/// - either: map from signature to 32/64 bit execution functions
///  or: just a single 32/64execution function (lui does not need signature)
pub struct InstructionSpec<InstrType> {
    pub instr_type: InstrType,
    pub exec_fns: enum(map, fn)
    // pub signature: Option<u32>,
    // exec fns
    // mnemonic?
}

// Use macros to build InstructionSpec for each instruction

// To decode: read opcode, map to instruction spec. if
// signature present, use InstrType::mask(instr) to 
// get signature, and 

#[derive(Clone, Copy)]
pub struct Itype {
    pub rs1: u8,
    pub imm: u16,
    pub rd: u8,
}

#[derive(Clone, Copy)]
pub struct Ishtype {
    pub rs1: u8,
    pub shamt: u8,
    pub rd: u8,
}

#[derive(Clone, Copy)]
pub struct Stype {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u16,
}

#[derive(Clone, Copy)]
pub struct Btype {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u16,
}

#[derive(Clone, Copy)]
pub struct Utype {
    pub rd: u8,
    pub imm: u32,
}

#[derive(Clone, Copy)]
pub struct Jtype {
    pub rd: u8,
    pub imm: u32,
}

impl From<u32> for Rtype {
    fn from(instr: u32) -> Rtype {
        Rtype {
            rs1: rs1!(instr),
            rs2: rs2!(instr),
            rd: rd!(instr),
        }
    }
}

impl From<u32> for Itype {
    fn from(instr: u32) -> Itype {
        Itype {
            rs1: rs1!(instr),
            imm: imm_itype!(instr),
            rd: rd!(instr),
        }
    }
}

impl From<u32> for Stype {
    fn from(instr: u32) -> Stype {
        Stype {
            rs1: rs1!(instr),
            rs2: rs2!(instr),
            imm: imm_stype!(instr),
        }
    }
}

impl From<u32> for Btype {
    fn from(instr: u32) -> Btype {
        Btype {
            rs1: rs1!(instr),
            rs2: rs2!(instr),
            imm: imm_btype!(instr),
        }
    }
}

impl From<u32> for Utype {
    fn from(instr: u32) -> Utype {
        Utype {
            rd: rd!(instr),
            imm: lui_u_immediate!(instr),
        }
    }
}

impl From<u32> for Jtype {
    fn from(instr: u32) -> Jtype {
        Jtype {
            rd: rd!(instr),
            imm: jal_offset!(instr),
        }
    }
}
