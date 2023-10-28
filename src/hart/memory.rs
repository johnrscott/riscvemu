use thiserror::Error;
use std::collections::HashMap;

/// Word sizes defined in the RISC-V specification
pub enum Wordsize {
    Byte,
    Halfword,
    Word,
    Doubleword,
}

impl Wordsize {
    fn width(&self) -> u8 {
	match self {
	    Wordsize::Byte => 1,
	    Wordsize::Halfword => 2,
	    Wordsize::Word => 4,
	    Wordsize::Doubleword => 8,
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

    pub fn new(xlen: Xlen) -> Self {
	Self {
	    xlen, ..Self::default()
	}
    }
    
    pub fn write(&mut self, addr: u64, value: u64, word_size: Wordsize) -> Result<(), WriteError> {
	let addr = wrap_address(addr, self.xlen);
	let write_width = word_size.width().try_into().unwrap();
	write_word(&mut self.data, addr, write_width, value);
	Ok(())
    }
    
    pub fn read(&self, addr: u64, word_size: Wordsize) -> Result<u64, ReadError> {
	let addr = wrap_address(addr, self.xlen);
	let read_width = word_size.width().try_into().unwrap();
	let result = read_word(&self.data, addr, read_width);
	Ok(result.try_into().unwrap())
    }

}

#[cfg(test)]
mod tests {

    use super::*;
    
    /// Just test a few of each type of read
    #[test]
    fn memory_zero_initialised() {
	let mem = Memory::default();
	for addr in (0..100).step_by(11) {
	    assert_eq!(mem.read(addr, Wordsize::Byte).unwrap(), 0);
	    assert_eq!(mem.read(addr, Wordsize::Halfword).unwrap(), 0);
	    assert_eq!(mem.read(addr, Wordsize::Word).unwrap(), 0);
	    assert_eq!(mem.read(addr, Wordsize::Doubleword).unwrap(), 0);
	}
    }

    #[test]
    fn byte_write_then_read() {
	let mut mem = Memory::default();
	for addr in (0..100).step_by(11) {
	    let value = 17*addr;
	    mem.write(addr, value, Wordsize::Byte).unwrap();
	    assert_eq!(mem.read(addr, Wordsize::Byte).unwrap(), 0xff & value);
	    // Check write did not spill into next byte
	    assert_eq!(mem.read(addr+1, Wordsize::Byte).unwrap(), 0);
	}
    }

    #[test]
    fn halfword_write_then_read() {
	let mut mem = Memory::default();
	for addr in (0..100).step_by(11) {
	    let value = 17*addr  + 0x4ff0;
	    mem.write(addr, value, Wordsize::Halfword).unwrap();
	    assert_eq!(mem.read(addr, Wordsize::Halfword).unwrap(), 0xffff & value);
	    // Check write did not spill into next byte
	    assert_eq!(mem.read(addr+2, Wordsize::Halfword).unwrap(), 0);
	}
    }

    #[test]
    fn word_write_then_read() {
	let mut mem = Memory::default();
	for addr in (0..100).step_by(11) {
	    let value = 17*addr + 0x9e4f_3ff0;
	    mem.write(addr, value, Wordsize::Word).unwrap();
	    assert_eq!(mem.read(addr, Wordsize::Word).unwrap(), 0xffffffff & value);
	    // Check write did not spill into next byte
	    assert_eq!(mem.read(addr+4, Wordsize::Word).unwrap(), 0);
	}
    }

    #[test]
    fn doubleword_write_then_read() {
	let mut mem = Memory::default();
	for addr in (0..100).step_by(11) {
	    let value = 17*addr + 0x12ae_abf0;
	    mem.write(addr, value, Wordsize::Doubleword).unwrap();
	    assert_eq!(mem.read(addr, Wordsize::Doubleword).unwrap(), 0xffffffff & value);
	    // Check write did not spill into next byte
	    assert_eq!(mem.read(addr+8, Wordsize::Doubleword).unwrap(), 0);
	}
    }

    #[test]
    fn check_32bit_memory_wrap() {
	let mut mem = Memory::default();
	let value = 0x0403_0201;
	let addr = 0xffff_ffff;
	mem.write(addr, value, Wordsize::Word).unwrap();
	assert_eq!(mem.read(addr, Wordsize::Byte).unwrap(), 1);
	assert_eq!(mem.read(0, Wordsize::Byte).unwrap(), 2);
	assert_eq!(mem.read(1, Wordsize::Byte).unwrap(), 3);
	assert_eq!(mem.read(1, Wordsize::Byte).unwrap(), 4);
    }
    
    
    
}
