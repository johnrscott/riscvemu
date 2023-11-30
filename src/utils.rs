use std::ops::{BitAnd, Shl, Shr};

use num::Integer;

/// Make an n_bits-long mask (all ones)
pub fn mask<T>(n_bits: T) -> T
where
    T: Integer + Shl<Output = T>,
{
    (T::one() << n_bits) - T::one()
}

/// Obtain value[end:start] (verilog notation) from value
pub fn extract_field<T>(value: T, end: T, start: T) -> T
where
    T: Copy + Integer + Shl<Output = T> + Shr<Output = T> + BitAnd<Output = T>,
{
    mask(end - start + T::one()) & (value >> start)
}

pub fn interpret_u32_as_signed(value: u32) -> i32 {
    i32::from_ne_bytes(value.to_ne_bytes())
}

pub fn interpret_i32_as_unsigned(value: i32) -> u32 {
    u32::from_ne_bytes(i32::from(value).to_ne_bytes())
}

/// Take an unsigned value (u8, u16 or u32), and a bit position for the
/// sign bit, and copy the value of the sign bit into all the higher bits
/// of the u32.
pub fn sign_extend<T: Into<u32>>(value: T, sign_bit_position: u32) -> u32 {
    let value: u32 = value.into();
    let sign_bit = 1 & (value >> sign_bit_position);
    if sign_bit == 1 {
        let sign_extension = 0xffff_ffff - mask(sign_bit_position);
        value | sign_extension
    } else {
        value
    }
}
