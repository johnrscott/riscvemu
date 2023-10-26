macro_rules! opcode {
    ($instr:expr) => {
        extract_field!($instr, 6, 0)
    };
}

macro_rules! funct3 {
    ($instr:expr) => {
        extract_field!($instr, 14, 12)
    };
}

macro_rules! funct7 {
    ($instr:expr) => {
        extract_field!($instr, 31, 25)
    };
}

/// The shift amount for RV32I and RV64I is stored in the lower
/// portion of what would be the imm field in an itype instruction.
/// For 32-bit operation, the field is 5 bits, and for 64-bit
/// operation it is 6 bits. This function returns a u8 that includes
/// the full 6-bit field, which is important for checking whether
/// the field is valid in 64-bit mode.
macro_rules! shamt {
    ($instr:expr) => {{
        let shamt: u8 = extract_field!($instr, 25, 20).try_into().unwrap();
	shamt
    }};
}

/// The flag for being an arithmetic (instead of logical)
/// right shift is stored in bit 30 of the instruction.
/// Used to distinguish sra, srl, srai, srli.
macro_rules! is_arithmetic_shift {
    ($instr:expr) => {extract_field!($instr, 30, 30) == 1}
}


macro_rules! imm_itype {
    ($instr:expr) => {{
	let imm: u16 = extract_field!($instr, 31, 20).try_into().unwrap();
	imm
    }};
}

macro_rules! imm_btype {
    ($instr:expr) => {{
        let imm12 = extract_field!($instr, 31, 31);
    	let imm11 = extract_field!($instr, 7, 7);
	let imm10_5 = extract_field!($instr, 30, 25);
	let imm4_1 = extract_field!($instr, 11, 8);
	(imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1)
    }};
}

macro_rules! rd {
    ($instr:expr) => {{
        let rd: u8 = extract_field!($instr, 11, 7).try_into().unwrap();
        rd
    }};
}

macro_rules! rs1 {
    ($instr:expr) => {{
        let rs1: u8 = extract_field!($instr, 19, 15).try_into().unwrap();
        rs1
    }};
}

macro_rules! rs2 {
    ($instr:expr) => {{
        let rs2: u8 = extract_field!($instr, 24, 20).try_into().unwrap();
        rs2
    }};
}


macro_rules! lui_u_immediate {
    ($instr:expr) => {
        extract_field!($instr, 31, 12)
    };
}
