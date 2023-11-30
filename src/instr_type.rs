use crate::utils::extract_field;

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
        rs1: rs1(instr),
        rs2: rs2(instr),
        rd: rd(instr),
    }
}

pub fn decode_itype(instr: u32) -> Itype {
    Itype {
        rs1: rs1(instr),
        imm: imm_itype(instr),
        rd: rd(instr),
    }
}

pub fn decode_stype(instr: u32) -> SBtype {
    SBtype {
        rs1: rs1(instr),
        rs2: rs2(instr),
        imm: imm_stype(instr),
    }
}

pub fn decode_btype(instr: u32) -> SBtype {
    SBtype {
        rs1: rs1(instr),
        rs2: rs2(instr),
        imm: imm_btype(instr).try_into().unwrap(),
    }
}

pub fn decode_utype(instr: u32) -> UJtype {
    UJtype {
        rd: rd(instr),
        imm: lui_u_immediate(instr),
    }
}

pub fn decode_jtype(instr: u32) -> UJtype {
    UJtype {
        rd: rd(instr),
        imm: jal_offset(instr),
    }
}

/// Makes a function called field_name which gets that field from a
/// 32-bit instruction. Specify the output type using field_type
/// (generally picked to be the smallest type which will fit the
/// field). The function will extract instr[end:start] (verilog
/// notation).
macro_rules! make_field_getter {
    ($field_name:ident, $field_type:ty, $end:expr, $start:expr) => {
        /// Get the field $field_name from instruction (bits
        /// instr[$end:$start] in verilog notation).
        fn $field_name(instr: u32) -> $field_type {
            extract_field(instr, $end, $start).try_into().unwrap()
        }
    };
}

make_field_getter!(rd, u8, 11, 7);
make_field_getter!(rs1, u8, 19, 15);
make_field_getter!(rs2, u8, 24, 20);
make_field_getter!(lui_u_immediate, u32, 31, 12);
make_field_getter!(imm_itype, u16, 31, 20);

/// Get the jal instruction offset field from an instruction
fn jal_offset(instr: u32) -> u32 {
    let imm20 = extract_field(instr, 31, 31);
    let imm19_12 = extract_field(instr, 19, 12);
    let imm11 = extract_field(instr, 20, 20);
    let imm10_1 = extract_field(instr, 30, 21);
    (imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1)
}

/// Get the immediate field in an S-type instruction
fn imm_stype(instr: u32) -> u16 {
    let imm11_5: u16 = extract_field(instr, 31, 25).try_into().unwrap();
    let imm4_0: u16 = extract_field(instr, 11, 7).try_into().unwrap();
    (imm11_5 << 5) | imm4_0
}

/// Get the immediate field in an B-type instruction
fn imm_btype(instr: u32) -> u16 {
    let imm12 = extract_field(instr, 31, 31);
    let imm11 = extract_field(instr, 7, 7);
    let imm10_5 = extract_field(instr, 30, 25);
    let imm4_1 = extract_field(instr, 11, 8);
    let imm = (imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1);
    imm.try_into().unwrap()
}
