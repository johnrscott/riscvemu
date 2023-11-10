use super::fields::*;

#[derive(Debug, Clone, Copy)]
pub struct Rtype {
    pub rs1: u8,
    pub rs2: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct Itype {
    pub rs1: u8,
    pub imm: u16,
    pub rd: u8,
}

/// A specialisation of I-type for shift
/// functions which store a shift amount, and
/// use bit 30 to indicate the type shift.
#[derive(Debug, Clone, Copy)]
pub struct Ishtype {
    pub rs1: u8,
    pub shamt: u8,
    pub rd: u8,
}

#[derive(Debug, Clone, Copy)]
pub struct SBtype {
    pub rs1: u8,
    pub rs2: u8,
    pub imm: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct UJtype {
    pub rd: u8,
    pub imm: u32,
}

pub fn decode_rtype(instr: u32) -> Rtype {
    Rtype {
        rs1: rs1!(instr),
        rs2: rs2!(instr),
        rd: rd!(instr),
    }
}

pub fn decode_itype(instr: u32) -> Itype {
    Itype {
        rs1: rs1!(instr),
        imm: imm_itype!(instr),
        rd: rd!(instr),
    }
}

pub fn decode_stype(instr: u32) -> SBtype {
    SBtype {
        rs1: rs1!(instr),
        rs2: rs2!(instr),
        imm: imm_stype!(instr),
    }
}

pub fn decode_btype(instr: u32) -> SBtype {
    SBtype {
        rs1: rs1!(instr),
        rs2: rs2!(instr),
        imm: imm_btype!(instr).try_into().unwrap(),
    }
}

pub fn decode_utype(instr: u32) -> UJtype {
    UJtype {
        rd: rd!(instr),
        imm: lui_u_immediate!(instr),
    }
}

pub fn decode_jtype(instr: u32) -> UJtype {
    UJtype {
        rd: rd!(instr),
        imm: jal_offset!(instr),
    }
}
