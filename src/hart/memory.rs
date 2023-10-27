use thiserror::Error;
use std::collections::HashMap;

/// Word sizes defined in the RISC-V specification
pub enum WordSize {
    Byte,
    Halfword,
    Word,
    Doubleword,
}

impl WordSize {
    fn width(&self) -> u8 {
	match self {
	    WordSize::Byte => 1,
	    WordSize::Halfword => 2,
	    WordSize::Word => 4,
	    WordSize::Doubleword => 8,
	}
    }
}

/// The register and address-space width in RISC-V
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum Xlen {
    #[default]
    Xlen32,
    Xlen64,
}

/// RISC-V Hart Memory
///
/// The basic memory model is described in section
/// 1.4 of the RISC-V unprivileged reference.
///
///
#[derive(Debug, Default)]
pub struct Memory {
    xlen: Xlen,
    data: HashMap<u64, u8>,
}

#[derive(Error, Debug)]
pub enum ReadError {
    #[error("wrong address size")]
    WrongAddressSize,
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("wrong address size")]
    WrongAddressSize,
}

fn read_byte(byte_map: &HashMap<u64, u8>, addr: u64) -> u64 {
    u64::from(*byte_map.get(&addr).unwrap_or(&0))
}

fn read_word(byte_map: &HashMap<u64, u8>, addr: u64, num_bytes: u64) -> u64 {
    let mut value = 0;
    for n in 0..num_bytes {
	let byte_n = read_byte(byte_map, addr+n);
	value |= byte_n << 8*n;
    }
    value
}
    
fn write_byte(byte_map: &mut HashMap<u64, u8>, addr: u64, value: u8) {
    if value == 0 {
	byte_map.remove(&addr);
    } else {
	byte_map.insert(addr, value);
    }
}

fn write_word(byte_map: &mut HashMap<u64, u8>, addr: u64, num_bytes: u64, value: u64,) {
    for n in 0..num_bytes {
	let byte_n = 0xff & (value >> 8*n);
	write_byte(byte_map, addr+n, byte_n.try_into().unwrap());
    }
}

fn wrap_address(addr: u64, xlen: Xlen) -> u64 {
    match xlen {
	Xlen::Xlen32 => 0xffffffff & addr,
	Xlen::Xlen64 => addr,
    }
}

impl Memory {

    pub fn write(&mut self, addr: u64, value: u64, word_size: WordSize) -> Result<(), WriteError> {
	let addr = wrap_address(addr, self.xlen);
	let write_width = word_size.width().try_into().unwrap();
	write_word(&mut self.data, addr, write_width, value);
	Ok(())
    }
    
    pub fn read(&self, addr: u64, word_size: WordSize) -> Result<u64, ReadError> {
	let addr = wrap_address(addr, self.xlen);
	let read_width = word_size.width().try_into().unwrap();
	let result = read_word(&self.data, addr, read_width);
	Ok(result.try_into().unwrap())
    }

}
