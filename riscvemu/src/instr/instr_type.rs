#[derive(Clone, Copy)]
pub enum InstrType {
    Rtype {
	pub rs1: u8,
	pub rs2: u8,
	pub rd: u8,
    },
    Itype {
	pub rs1: u8,
	pub imm: u16,
	pub rd: u8,
    },
    /// A specialisation of I-type for shift
    /// functions which store a shift amount, and
    /// use bit 30 to indicate the type shift.
    Ishtype {
	pub rs1: u8,
	pub shamt: u8,
	pub rd: u8,
    },
    Stype {
	pub rs1: u8,
	pub rs2: u8,
	pub imm: u16,
    },
    Btype {
	pub rs1: u8,
	pub rs2: u8,
	pub imm: u16,
    },
    Utype {
	pub rd: u8,
	pub imm: u32,
    },
    Jtype {
	pub rd: u8,
	pub imm: u32,
    }
}



impl InstrType {

    /// Mask out the portion of the instruction not in the
    /// signature, leaving only the part which is used to identify
    /// the instruction (the signature)
    pub fn mask(&self, instr: u32) -> u32 {
	match self {
	    Self::Rtype { .. } => (mask!(7) << 25 | mask!(3) << 12 | mask!(7)) & instr,
	    
	}
	
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
pub struct InstructionSpec {
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
