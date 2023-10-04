use crate::fields;
use std::collections::HashMap;
use std::fmt;

#[derive(Debug)]
pub struct Memory {
    data: HashMap<usize, u8>,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    pub fn write_byte(&mut self, addr: usize, value: u8) {
        self.data.insert(addr, value);
    }

    pub fn read_byte(&self, addr: usize) -> u8 {
        if let Some(value) = self.data.get(&addr) {
            *value
        } else {
            0
        }
    }

    pub fn write_halfword(&mut self, addr: usize, value: u16) {
        self.write_byte(addr, fields::extract_bit_range(value.into(), 0, 8) as u8);
        self.write_byte(
            addr + 1,
            fields::extract_bit_range(value.into(), 8, 8) as u8,
        )
    }

    pub fn read_halfword(&self, addr: usize) -> u16 {
        let byte_0 = self.read_byte(addr);
        let byte_1 = self.read_byte(addr + 1);
        u16::from_le_bytes([byte_0, byte_1])
    }

    pub fn write_word(&mut self, addr: usize, value: u32) {
        self.write_halfword(addr, fields::extract_bit_range(value.into(), 0, 16) as u16);
        self.write_halfword(
            addr + 2,
            fields::extract_bit_range(value.into(), 16, 16) as u16,
        )
    }

    pub fn read_word(&self, addr: usize) -> u32 {
        let halfword_0 = self.read_halfword(addr);
        let halfword_1 = self.read_halfword(addr + 2);
        ((halfword_1 as u32) << 16) | halfword_0 as u32
    }

    pub fn write_doubleword(&mut self, addr: usize, value: u64) {
        self.write_word(addr, fields::extract_bit_range(value.into(), 0, 32) as u32);
        self.write_word(
            addr + 4,
            fields::extract_bit_range(value.into(), 32, 32) as u32,
        )
    }

    pub fn read_doubleword(&self, addr: usize) -> u64 {
        let word_0 = self.read_byte(addr);
        let word_1 = self.read_byte(addr + 4);
        ((word_1 as u64) << 32) | word_0 as u64
    }
}

impl fmt::Display for Memory {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Memory {{")?;
        for addr in (0..32).step_by(4) {
            let value = self.read_word(addr);
            writeln!(f, " {addr:02x}: {value:08x}")?;
        }
        writeln!(f, "}}")
    }
}
