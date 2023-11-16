use thiserror::Error;

use super::memory::Xlen;

#[derive(Debug, Default)]
pub struct Registers {
    xlen: Xlen,
    registers: [u64; 32],
}

#[derive(Error, PartialEq, Eq, Debug)]
pub enum RegisterReadError {
    #[error("read register index exceeds 31")]
    InvalidRegister,
}

#[derive(Error, PartialEq, Eq, Debug)]
pub enum RegisterWriteError {
    #[error("write register index exceeds 31")]
    InvalidRegister,
    #[error("attempted to write value larger than 32 bits to 32-bit register")]
    InvalidValue,
}

fn value_invalid(value: u64, xlen: Xlen) -> bool {
    xlen == Xlen::Xlen32 && value > 0xffff_ffff
}

impl Registers {
    pub fn new(xlen: Xlen) -> Self {
        Self {
            xlen,
            ..Default::default()
        }
    }

    pub fn write(&mut self, which: usize, value: u64) -> Result<(), RegisterWriteError> {
        if value_invalid(value, self.xlen) {
            Err(RegisterWriteError::InvalidValue)
        } else if which > 31 {
            return Err(RegisterWriteError::InvalidRegister);
        } else {
            if which != 0 {
                self.registers[which] = value;
            }
            Ok(())
        }
    }

    pub fn read(&self, which: usize) -> Result<u64, RegisterReadError> {
        if which > 31 {
            Err(RegisterReadError::InvalidRegister)
        } else {
            Ok(self.registers[which])
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn check_registers_initialised_to_zero() {
        let reg = Registers::default();
        for n in 0..31 {
            assert_eq!(reg.read(n).unwrap(), 0)
        }
    }

    #[test]
    fn check_register_read_out_of_bounds() {
        let reg = Registers::default();
        let result = reg.read(32);
        assert_eq!(result, Err(RegisterReadError::InvalidRegister));
    }

    #[test]
    fn check_register_write_out_of_bounds() {
        let mut reg = Registers::default();
        let result = reg.write(32, 12);
        assert_eq!(result, Err(RegisterWriteError::InvalidRegister));
    }

    #[test]
    fn check_write_then_read() {
        let mut reg = Registers::default();
        // Note how the write to x0 is zero
        for n in 0..31 {
            let value = (2 * n).try_into().unwrap();
            reg.write(n, value).unwrap();
            assert_eq!(reg.read(n).unwrap(), value);
        }
    }

    #[test]
    fn check_write_then_read_x0() {
        let mut reg = Registers::default();
        let value = 0x3423;
        reg.write(0, value).unwrap();
        assert_eq!(reg.read(0).unwrap(), 0);
    }

    #[test]
    fn check_invalid_value_in_32bit() {
        let mut reg = Registers::default();
        let value = 0x1_0000_0000;
        let result = reg.write(10, value);
        assert_eq!(result, Err(RegisterWriteError::InvalidValue));
    }

    #[test]
    fn check_valid_value_in_64bit() {
        let mut reg = Registers::new(Xlen::Xlen64);
        let value = 0x1_0000_0000;
        reg.write(10, value).unwrap();
        assert_eq!(reg.read(10).unwrap(), value);
    }
}
