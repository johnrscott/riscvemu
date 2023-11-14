use std::mem;

pub fn n_bit_mask(num_bits: u64) -> u64 {
    (1 << num_bits) - 1
}

pub fn extract_bit_range(instr: u64, start: u64, width: u64) -> u64 {
    let end = start + width - 1;
    if end >= 64 {
        panic!("This field [{end}:{start}] does not fall within a u64");
    }
    n_bit_mask(width) & (instr >> start)
}

pub fn opcode(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 0, 7) as u8
}

pub fn rd(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 7, 5) as u8
}

pub fn funct3(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 12, 3) as u8
}

pub fn rs1(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 15, 5) as u8
}

pub fn rs2(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 20, 5) as u8
}

pub fn funct7(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 25, 7) as u8
}

pub fn imm_itype(instr: u32) -> i16 {
    let mut unsigned = extract_bit_range(instr.into(), 20, 12) as u16;
    let sign_bit = 1 & (unsigned >> 11);
    if sign_bit == 1 {
        unsigned = 0xf000u16 | unsigned;
    }
    unsafe { mem::transmute(unsigned) }
}

pub fn imm_stype(instr: u32) -> i16 {
    let imm11_5 = funct7(instr) as u16;
    let imm4_0 = rd(instr) as u16;
    let mut unsigned = (imm11_5 << 4) | imm4_0;
    let sign_bit = 1 & (unsigned >> 11);
    if sign_bit == 1 {
        unsigned = 0xf000u16 | unsigned;
    }
    unsafe { mem::transmute(unsigned) }
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

