/// Make a bit-mask of n bits using mask!(n)
#[macro_export]
macro_rules! mask {
    ($n:expr) => {
        (1 << $n) - 1
    };
}
pub use mask;

/// Mask a value to n least significant bits and
/// shift it left by s bits
#[macro_export]
macro_rules! mask_and_shift {
    ($val:expr, $m:expr, $s:expr) => {
        (mask!($m) & $val) << $s
    };
}
pub use mask_and_shift;

/// Return val[end:start]
#[macro_export]
macro_rules! extract_field {
    ($val:expr, $end:expr, $start:expr) => {{
        mask!($end - $start + 1) & ($val >> $start)
    }};
}
pub use extract_field;

#[macro_export]
macro_rules! opcode {
    ($instr:expr) => {
        extract_field!($instr, 6, 0)
    };
}
pub use opcode;

#[macro_export]
macro_rules! funct3 {
    ($instr:expr) => {
        extract_field!($instr, 14, 12)
    };
}
pub use funct3;

#[macro_export]
macro_rules! funct7 {
    ($instr:expr) => {
        extract_field!($instr, 31, 25)
    };
}
pub use funct7;

/// Return the offset including the least-significant
/// zero (i.e. 21 bits long)
#[macro_export]
macro_rules! jal_offset {
    ($instr:expr) => {{
        let imm20 = extract_field!($instr, 31, 31);
	let imm19_12 = extract_field!($instr, 19, 12);
	let imm11 = extract_field!($instr, 20, 20);
    	let imm10_1 = extract_field!($instr, 30, 21);
	(imm20 << 20) | (imm19_12 << 12) | (imm11 << 11) | (imm10_1 << 1)
    }};
}
pub use jal_offset;

/// The shift amount for RV32I and RV64I is stored in the lower
/// portion of what would be the imm field in an itype instruction.
/// For 32-bit operation, the field is 5 bits, and for 64-bit
/// operation it is 6 bits. This function returns a u8 that includes
/// the full 6-bit field, which is important for checking whether
/// the field is valid in 64-bit mode.
#[macro_export]
macro_rules! shamt {
    ($instr:expr) => {{
        let shamt: u8 = extract_field!($instr, 25, 20).try_into().unwrap();
	shamt
    }};
}
pub use shamt;

/// The flag for being an arithmetic (instead of logical)
/// right shift is stored in bit 30 of the instruction.
/// Used to distinguish sra, srl, srai, srli.
#[macro_export]
macro_rules! is_arithmetic_shift {
    ($instr:expr) => {extract_field!($instr, 30, 30) == 1}
}
pub use is_arithmetic_shift;

#[macro_export]
macro_rules! imm_itype {
    ($instr:expr) => {{
	let imm: u16 = extract_field!($instr, 31, 20).try_into().unwrap();
	imm
    }};
}
pub use imm_itype;

#[macro_export]
macro_rules! imm_btype {
    ($instr:expr) => {{
        let imm12 = extract_field!($instr, 31, 31);
    	let imm11 = extract_field!($instr, 7, 7);
	let imm10_5 = extract_field!($instr, 30, 25);
	let imm4_1 = extract_field!($instr, 11, 8);
	(imm12 << 12) | (imm11 << 11) | (imm10_5 << 5) | (imm4_1 << 1)
    }};
}
pub use imm_btype;

#[macro_export]
macro_rules! rd {
    ($instr:expr) => {{
        let rd: u8 = extract_field!($instr, 11, 7).try_into().unwrap();
        rd
    }};
}
pub use rd;

#[macro_export]
macro_rules! rs1 {
    ($instr:expr) => {{
        let rs1: u8 = extract_field!($instr, 19, 15).try_into().unwrap();
        rs1
    }};
}
pub use rs1;

#[macro_export]
macro_rules! rs2 {
    ($instr:expr) => {{
        let rs2: u8 = extract_field!($instr, 24, 20).try_into().unwrap();
        rs2
    }};
}
pub use rs2;

#[macro_export]
macro_rules! lui_u_immediate {
    ($instr:expr) => {
        extract_field!($instr, 31, 12)
    };
}
pub use lui_u_immediate;
