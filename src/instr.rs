/// Make a bit-mask of n bits using mask!(n)
macro_rules! mask {
    ($n:expr) => {
        (1 << $n) - 1
    };
}
pub(crate) use mask;

/// Mask a value to n least significant bits and
/// shift it left by s bits
macro_rules! mask_and_shift {
    ($val:expr, $m:expr, $s:expr) => {
        (mask!($m) & $val) << $s
    };
}
pub(crate) use mask_and_shift;

/// Return val[end:start]
macro_rules! extract_field {
    ($val:expr, $end:expr, $start:expr) => {{
	mask!($end - $start + 1) & ($val >> $start)
    }}
}
pub(crate) use extract_field;

/// Make an I-type instruction
macro_rules! itype {
    ($imm:expr, $rs1:expr, $funct3:expr, $rd:expr, $opcode:expr) => {
        mask_and_shift!($imm, 12, 20)
            | mask_and_shift!($rs1, 5, 15)
            | mask_and_shift!($funct3, 3, 12)
            | mask_and_shift!($rd, 5, 7)
            | mask_and_shift!($opcode, 7, 0)
    };
}
pub(crate) use itype;

/// Make an U- or J-type instruction (if you are making
/// a J-type instruction, make sure to construct the
/// immediate field correctly using jtype_imm_fields)
macro_rules! ujtype {
    ($imm:expr, $rd:expr, $opcode:expr) => {
        mask_and_shift!($imm, 20, 12)
            | mask_and_shift!($rd, 5, 7)
            | mask_and_shift!($opcode, 7, 0)
    };
}
pub(crate) use ujtype;

/// Make an R- or S-type instruction. These instructions
/// have the same number of fields of the same size. The meaning
/// of a and b is:
///
/// R-type: a = funct7, b = rd
/// S-type: a = imm[11:5], b = imm[4:0]
macro_rules! rstype {
    ($a:expr, $rs2:expr, $rs1:expr, $funct3:expr, $b:expr, $opcode:expr) => {
        mask_and_shift!($a, 7, 25)
            | mask_and_shift!($rs2, 5, 20)
            | mask_and_shift!($rs1, 5, 15)
            | mask_and_shift!($funct3, 3, 12)
            | mask_and_shift!($b, 5, 7)
            | mask_and_shift!($opcode, 7, 0)
    };
}
pub(crate) use rstype;

/// Convert a RISC-V register name (e.g. x3) to the register
/// value (e.g. 3)
///
///
pub fn reg_num_impl(reg_name: &str) -> Result<u32, &'static str> {
    if reg_name.len() != 2 && reg_name.len() != 3 {
        return Err("register name must be exactly two or three characters");
    }
    let mut characters = reg_name.chars();
    if characters.next().unwrap() != 'x' {
        return Err("register name must begin with x");
    }
    let n = characters
	.collect::<String>()
        .parse::<u32>()
        .expect("Final one or two digits of register name should be numbers");
    Ok(n)
}

macro_rules! reg_num {
    ($reg:expr) => {
	reg_num_impl(std::stringify!($reg))?
    }
}
pub(crate) use reg_num;

macro_rules! imm_as_u32 {
    ($imm:expr) => {{
	let imm_as_u32: u32 = unsafe { std::mem::transmute($imm) };
	imm_as_u32
    }}
}
pub(crate) use imm_as_u32;

macro_rules! itype_instr {
    ($instruction:ident, $funct3:expr, $opcode:expr) => {
	macro_rules! $instruction {
	    ($rd:ident, $rs1:expr, $imm:expr) => {{
		let rd = reg_num!($rd);
		let rs1 = reg_num!($rs1);
		let imm = imm_as_u32!($imm);
		itype!(imm, rs1, $funct3, rd, $opcode)
	    }};
	}
	pub(crate) use $instruction;
    }
}

/// Here, upper is the only special value, which is always zero
/// apart from in srai, where it is 0b0100000. 
macro_rules! shift_instr {
    ($instruction:ident, $upper:expr, $funct3:expr, $opcode:expr) => {
	macro_rules! $instruction {
	    ($rd:ident, $rs1:expr, $imm:expr) => {{
		let rd = reg_num!($rd);
		let rs1 = reg_num!($rs1);
		let imm = shifts_imm_field!($imm, $upper);
		itype!(imm, rs1, $funct3, rd, $opcode)
	    }};
	}
	pub(crate) use $instruction;
    }
}

macro_rules! rtype_instr {
    ($instruction:ident, $funct7:expr, $funct3:expr, $opcode:expr) => {
	macro_rules! $instruction {
	    ($rd:ident, $rs1:expr, $rs2:expr) => {{
		let rd = reg_num!($rd);
		let rs1 = reg_num!($rs1);
		let rs2 = reg_num!($rs2);		
		rstype!($funct7, rs2, rs1, $funct3, rd, $opcode)
	    }};
	}
	pub(crate) use $instruction;
    }
}

macro_rules! stype_instr {
    ($instruction:ident, $funct3:expr, $opcode:expr) => {
	macro_rules! $instruction {
	    ($rs2:expr, $rs1:expr, $imm:expr) => {{
		let rs1 = reg_num!($rs1);
		let rs2 = reg_num!($rs2);
		let imm = imm_as_u32!($imm);
		let imm11_5 = extract_field!(imm, 11, 5);
		let imm4_0 = extract_field!(imm, 4, 0);
		rstype!(imm11_5, rs2, rs1, $funct3, imm4_0, $opcode)
	    }};
	}
	pub(crate) use $instruction;
    }
}

/// The shift-by-immediate instructions use I-type,
/// but with a special encoding of the immediate that
/// uses the lower 5 bits for the shift amount (shamt)
/// and the upper 7 bits to distinguish between arithmetical
/// and logical right shift
macro_rules! shifts_imm_field {
    ($shamt:expr, $upper:expr) => {{
	let shamt = extract_field!($shamt, 4, 0);
	($upper << 5) | shamt
    }}
}
pub(crate) use shifts_imm_field;


/// Takes an immediate and shuffles it into the
/// format required for the 20-bit field of the
/// U-type instruction (making it J-type)
macro_rules! jtype_imm_field {
    ($imm:expr) => {{
	let imm = imm_as_u32!($imm);
	let imm20 = extract_field!(imm, 20, 20);
	let imm19_12 = extract_field!(imm, 19, 12);
	let imm11 = extract_field!(imm, 11, 11);
	let imm10_1 = extract_field!(imm, 10, 1);
	(imm20 << 19) | (imm10_1 << 9) | (imm11 << 8) | imm19_12
    }}
}
pub(crate) use jtype_imm_field;

/// Returns (a, b) suitable for use with rstype for
/// the conditional branch instructions (btype)
macro_rules! btype_imm_fields {
    ($imm:expr) => {{
	let imm = imm_as_u32!($imm);
	let imm12 = extract_field!(imm, 12, 12);
	let imm11 = extract_field!(imm, 11, 11);
	let imm10_5 = extract_field!(imm, 10, 5);
	let imm4_1 = extract_field!(imm, 4, 1);
	let a = (imm12 << 6) | imm10_5;
	let b = (imm4_1 << 1) | imm11;
	println!("{a:b},{b:b}");
	(a,b)
    }}
}
pub(crate) use btype_imm_fields;

macro_rules! btype_instr {
    ($instruction:ident, $funct3:expr, $opcode:expr) => {
	macro_rules! $instruction {
	    ($rs1:expr, $rs2:expr, $imm:expr) => {{
		let rs1 = reg_num!($rs1);
		let rs2 = reg_num!($rs2);
		let (a, b) = btype_imm_fields!($imm);
		rstype!(a, rs2, rs1, $funct3, b, $opcode)
	    }};
	}
	pub(crate) use $instruction;
    }
}

macro_rules! jal {
    ($rd:expr, $imm:expr) => {{
	let rd = reg_num!($rd);
	let imm = jtype_imm_field!($imm);
	ujtype!(imm, rd, 0b1101111)
    }};
}
pub(crate) use jal;

/// Note: in these instructions (LUI and AUIPC), the immediate imm
/// is already the upper 20 bits that will be loaded -- it will not
/// be shifted up.
macro_rules! utype_instr {
    ($instruction:ident, $opcode:expr) => {
	macro_rules! $instruction {
	    ($rd:expr, $imm:expr) => {{
		let rd = reg_num!($rd);
		let imm = imm_as_u32!($imm);  
		ujtype!(imm, rd, $opcode)
	    }};
	}
	pub(crate) use $instruction;
    }
}


/// Instruction listing is in chapter 19 of RISC-V specification

//// R32I

utype_instr!(lui, 0b0110111);
utype_instr!(auipc, 0b0010111);
// jal is defined above
itype_instr!(jalr, 0b000, 0b1100111);

// Conditional branches
btype_instr!(beq, 0b000, 0b1100011);
btype_instr!(bne, 0b001, 0b1100011);
btype_instr!(blt, 0b100, 0b1100011);
btype_instr!(bge, 0b101, 0b1100011);
btype_instr!(bltu, 0b110, 0b1100011);
btype_instr!(bltu, 0b110, 0b1100011);
btype_instr!(bgeu, 0b111, 0b1100011);

// Loads
itype_instr!(lb, 0b000, 0b0000011);
itype_instr!(lh, 0b001, 0b0000011);
itype_instr!(lw, 0b010, 0b0000011);
itype_instr!(lbu, 0b100, 0b0000011);
itype_instr!(lhu, 0b101, 0b0000011);
// 64-bi
itype_instr!(lwu, 0b110, 0b0000011);
itype_instr!(ld, 0b011, 0b0000011);

// Stores
stype_instr!(sb, 0b000, 0b0100011);
stype_instr!(sh, 0b001, 0b0100011);
stype_instr!(sw, 0b010, 0b0100011);
// 64-bit
stype_instr!(sd, 0b011, 0b0100011);

/// Integer register-immediate instructions
itype_instr!(addi, 0b000, 0b0010011);
itype_instr!(slti, 0b010, 0b0010011);
itype_instr!(sltiu, 0b011, 0b0010011);
itype_instr!(xori, 0b100, 0b0010011);
itype_instr!(ori, 0b110, 0b0010011);
itype_instr!(andi, 0b111, 0b0010011);
// 64-bit
itype_instr!(addiw, 0b000, 0b0011011);

// Shift-by-immediate instructions. When using the 64-bit
// instruction set, these become 64-bit
shift_instr!(slli, 0b0000000, 0b001, 0b0010011);
shift_instr!(srli, 0b0000000, 0b101, 0b0010011);
shift_instr!(srai, 0b0100000, 0b101, 0b0010011);
// 64-bit
shift_instr!(slliw, 0b0000000, 0b001, 0b0011011);
shift_instr!(srliw, 0b0000000, 0b101, 0b0011011);
shift_instr!(sraiw, 0b0100000, 0b101, 0b0011011);

/// Integer register-register instructions
rtype_instr!(add, 0b0000000, 0b000, 0b0110011);
rtype_instr!(sub, 0b0100000, 0b000, 0b0110011);
rtype_instr!(sll, 0b0000000, 0b001, 0b0110011);
rtype_instr!(slt, 0b0000000, 0b010, 0b0110011);
rtype_instr!(sltu, 0b0000000, 0b011, 0b0110011);
rtype_instr!(xor, 0b0000000, 0b100, 0b0110011);
rtype_instr!(srl, 0b0000000, 0b101, 0b0110011);
rtype_instr!(sra, 0b0100000, 0b101, 0b0110011);
rtype_instr!(or, 0b0000000, 0b110, 0b0110011);
rtype_instr!(and, 0b0000000, 0b111, 0b0110011);
// 64-bit
rtype_instr!(addw, 0b0000000, 0b000, 0b0111011);
rtype_instr!(subw, 0b0100000, 0b000, 0b0111011);
rtype_instr!(sllw, 0b0000000, 0b001, 0b0111011);
rtype_instr!(srlw, 0b0000000, 0b101, 0b0111011);
rtype_instr!(sraw, 0b0100000, 0b101, 0b0111011);

// fence
// fence.i
// ecall
// ebreak

// csrrw
// csrrs
// csrrc
// csrrwi
// csrrsi
// csrrci
