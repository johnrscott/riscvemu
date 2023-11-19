use std::ops::{BitAnd, Shl, Shr};

use num::Integer;

/// Make an n_bits-long mask (all ones)
pub fn mask<T>(n_bits: T) -> T
where T: Integer + Shl<Output = T> {
    (T::one() << n_bits) - T::one()
}

/// Obtain value[end:start] (verilog notation) from value
pub fn extract_field<T>(value: T, end: T, start: T) -> T
where
    T: Copy + Integer + Shl<Output = T> + Shr<Output = T> + BitAnd<Output = T>,
{
    mask(end - start + T::one()) & (value >> start)
}

