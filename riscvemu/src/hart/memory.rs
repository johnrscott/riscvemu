use queues::*;
use std::collections::HashMap;
use thiserror::Error;

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
/// By default, memory is initialised for 32-bit mode
/// (xlen == 32). Values are still passed and returned
/// as u64, but an error is returned on read or write
/// if the address is larger than a 32-bit number.
///
/// Currently, all memory is considered as main memory
/// (there is no IO memory or vacant memory). When this
/// is added, the default behaviour will be to initialise
/// with a full main-memory address space, but the new
/// function will initialise with an all-vacant address
/// space. Error variants will be added for invalid
/// access to vacant, and the functions will be added
/// to create new address regions.
///
#[derive(Debug, Default)]
pub struct Memory {
    xlen: Xlen,
    data: HashMap<u64, u8>,
    stdout: Queue<char>,
}

#[derive(Error, PartialEq, Eq, Debug)]
pub enum ReadError {
    #[error("read address exceeds 0xffff_ffff in 32-bit mode")]
    InvalidAddress,
}

#[derive(Error, PartialEq, Eq, Debug)]
pub enum WriteError {
    #[error("read address exceeds to exceed 0xffff_ffff in 32-bit mode")]
    InvalidAddress,
}

fn wrap_address(addr: u64, xlen: Xlen) -> u64 {
    match xlen {
        Xlen::Xlen32 => 0xffffffff & addr,
        Xlen::Xlen64 => addr,
    }
}

fn read_byte(byte_map: &HashMap<u64, u8>, addr: u64, xlen: Xlen) -> u64 {
    let addr = wrap_address(addr, xlen);
    u64::from(*byte_map.get(&addr).unwrap_or(&0))
}

fn read_word(byte_map: &HashMap<u64, u8>, addr: u64, num_bytes: u64, xlen: Xlen) -> u64 {
    let mut value = 0;
    for n in 0..num_bytes {
        let byte_n = read_byte(byte_map, addr.wrapping_add(n), xlen);
        value |= byte_n << (8 * n);
    }
    value
}

fn address_invalid(addr: u64, xlen: Xlen) -> bool {
    xlen == Xlen::Xlen32 && addr > 0xffff_ffff
}

impl Memory {
    pub fn new(xlen: Xlen) -> Self {
        Self {
            xlen,
            ..Default::default()
        }
    }

    /// Return the current contents of the stdout buffer as a
    /// and also delete the contents of the buffer
    pub fn flush_stdout(&mut self) -> String {
        let mut stdout = String::new();
        while let Ok(ch) = self.stdout.remove() {
            stdout.push(ch);
        }
        stdout
    }

    fn write_byte(&mut self, addr: u64, value: u8, xlen: Xlen) {
        let addr = wrap_address(addr, xlen);
        // Char output device
        if addr == 0x3f8 {
            self.stdout
                .add(value as char)
                .expect("insert into queue should work");
        } else if value == 0 {
            self.data.remove(&addr);
        } else {
            self.data.insert(addr, value);
        }
    }

    fn write_word(&mut self, addr: u64, num_bytes: u64, value: u64, xlen: Xlen) {
        for n in 0..num_bytes {
            let byte_n = 0xff & (value >> (8 * n));
            self.write_byte(addr.wrapping_add(n), byte_n.try_into().unwrap(), xlen);
        }
    }

    pub fn write(&mut self, addr: u64, value: u64, word_size: Wordsize) -> Result<(), WriteError> {
        if address_invalid(addr, self.xlen) {
            Err(WriteError::InvalidAddress)
        } else {
            let write_width = word_size.width().try_into().unwrap();
            self.write_word(addr, write_width, value, self.xlen);
            Ok(())
        }
    }

    pub fn read(&self, addr: u64, word_size: Wordsize) -> Result<u64, ReadError> {
        if address_invalid(addr, self.xlen) {
            Err(ReadError::InvalidAddress)
        } else {
            let read_width = word_size.width().try_into().unwrap();
            let result = read_word(&self.data, addr, read_width, self.xlen);
            Ok(result)
        }
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
            let value = 17 * addr;
            mem.write(addr, value, Wordsize::Byte).unwrap();
            assert_eq!(mem.read(addr, Wordsize::Byte).unwrap(), 0xff & value);
            // Check write did not spill into next byte
            assert_eq!(mem.read(addr + 1, Wordsize::Byte).unwrap(), 0);
        }
    }

    #[test]
    fn halfword_write_then_read() {
        let mut mem = Memory::default();
        for addr in (0..100).step_by(11) {
            let value = 17 * addr + 0x4ff0;
            mem.write(addr, value, Wordsize::Halfword).unwrap();
            assert_eq!(mem.read(addr, Wordsize::Halfword).unwrap(), 0xffff & value);
            // Check write did not spill into next byte
            assert_eq!(mem.read(addr + 2, Wordsize::Halfword).unwrap(), 0);
        }
    }

    #[test]
    fn word_write_then_read() {
        let mut mem = Memory::default();
        for addr in (0..100).step_by(11) {
            let value = 17 * addr + 0x9e4f_3ff0;
            mem.write(addr, value, Wordsize::Word).unwrap();
            assert_eq!(mem.read(addr, Wordsize::Word).unwrap(), 0xffffffff & value);
            // Check write did not spill into next byte
            assert_eq!(mem.read(addr + 4, Wordsize::Word).unwrap(), 0);
        }
    }

    #[test]
    fn doubleword_write_then_read() {
        let mut mem = Memory::default();
        for addr in (0..100).step_by(11) {
            let value = 17 * addr + 0x12ae_abf0;
            mem.write(addr, value, Wordsize::Doubleword).unwrap();
            assert_eq!(
                mem.read(addr, Wordsize::Doubleword).unwrap(),
                0xffffffff & value
            );
            // Check write did not spill into next byte
            assert_eq!(mem.read(addr + 8, Wordsize::Doubleword).unwrap(), 0);
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
        assert_eq!(mem.read(2, Wordsize::Byte).unwrap(), 4);
    }

    #[test]
    fn check_64bit_memory_wrap() {
        let mut mem = Memory::new(Xlen::Xlen64);
        let value = 0x0403_0201;
        let addr = 0xffff_ffff_ffff_ffff;
        mem.write(addr, value, Wordsize::Word).unwrap();
        assert_eq!(mem.read(addr, Wordsize::Byte).unwrap(), 1);
        assert_eq!(mem.read(0, Wordsize::Byte).unwrap(), 2);
        assert_eq!(mem.read(1, Wordsize::Byte).unwrap(), 3);
        assert_eq!(mem.read(2, Wordsize::Byte).unwrap(), 4);
    }

    #[test]
    fn check_invalid_address_on_write() {
        let mut mem = Memory::default();
        let value = 0x0403_0201;
        let addr = 0x0_ffff_ffff;
        let result = mem.write(addr, value, Wordsize::Word);
        assert_eq!(result, Ok(()));
        let addr = 0x1_0000_0000;
        let result = mem.write(addr, value, Wordsize::Word);
        assert_eq!(result, Err(WriteError::InvalidAddress));
    }

    #[test]
    fn check_invalid_address_on_read() {
        let mem = Memory::default();
        let addr = 0x0_ffff_ffff;
        let result = mem.read(addr, Wordsize::Word);
        assert_eq!(result, Ok(0));
        let addr = 0x1_0000_0000;
        let result = mem.read(addr, Wordsize::Word);
        assert_eq!(result, Err(ReadError::InvalidAddress));
    }
}
