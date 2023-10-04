use std::fmt;

#[derive(Debug)]
pub struct RegisterFile {
    registers: [u64; 32],
}

impl RegisterFile {
    pub fn new() -> Self {
        Self { registers: [0; 32] }
    }

    pub fn set(&mut self, which: u8, value: u64) {
        if which >= 32 {
            panic!("Invalid attempt to address register {which}")
        }
        if which != 0 {
            self.registers[which as usize] = value
        }
    }

    pub fn get(&self, which: u8) -> u64 {
        if which >= 32 {
            panic!("Invalid attempt to address register {which}")
        }
        self.registers[which as usize]
    }
}

impl fmt::Display for RegisterFile {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "Non-zero registers {{")?;
        for n in 0..32 {
            let value = self.get(n);
            if value != 0 {
                writeln!(f, " x{n}: {}", self.get(n))?;
            }
        }
        writeln!(f, "}}")
    }
}
