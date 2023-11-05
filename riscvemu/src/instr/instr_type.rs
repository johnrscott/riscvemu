#[derive(Clone, Copy)]
pub struct Rtype {
    pub rs1: u8,
    pub rs2: u8,
    pub rd: u8,
}

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
