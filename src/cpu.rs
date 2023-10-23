use crate::register_file::RegisterFile;
use std::fmt;
use std::mem;

use crate::{fields, memory::Memory};

#[derive(Debug)]
pub struct Cpu {
    registers: RegisterFile,
    pc: u64,
    instructions: Memory,
    data: Memory,
}

impl Cpu {
    pub fn new() -> Self {
        Self {
            registers: RegisterFile::new(),
            pc: 0,
            instructions: Memory::new(),
            data: Memory::new(),
        }
    }

    pub fn set_program_counter(&mut self, pc: u64) {
        if pc % 4 != 0 {
            panic!("Program counter {pc} is not a multiple of 4");
        }
        self.pc = pc;
    }

    pub fn write_instruction(&mut self, pc: usize, instr: u32) {
        if pc % 4 != 0 {
            panic!("Program counter {pc} is not a multiple of 4");
        }
        self.instructions.write_word(pc, instr);
    }

    pub fn write_data(&mut self, addr: usize, value: u64, value_byte_width: usize) {
        match value_byte_width {
            1 => self.data.write_byte(addr, value.try_into().unwrap()),
            2 => self.data.write_halfword(addr, value.try_into().unwrap()),
            4 => self.data.write_word(addr, value.try_into().unwrap()),
            8 => self.data.write_doubleword(addr, value.try_into().unwrap()),
            _ => panic!("Invalid value_byte_width {value_byte_width}"),
        }
    }

    pub fn execute_instruction(&mut self) -> Result<(), &'static str> {
        // Read the next instruction
        let instr = self.instructions.read_word(self.pc.try_into().unwrap());

	// Check of zero instruction
	if instr == 0 {
	    return Err("Encountered illegal zero instruction");
	}
	
        // Check which instruction is being executed
        let op = fields::opcode(instr);
        match op {
            3 => {
                let rd = fields::rd(instr);
                let rs1 = fields::rs1(instr);
                let rs1_value = self.registers.get(rs1) as i64;
                let imm = fields::imm_itype(instr);
                let addr = rs1_value.wrapping_add(imm as i64) as usize;
                let mem_value = self.data.read_doubleword(addr);
                println!("ld x{rd} = *(x{rs1} + {imm})");
                self.registers.set(rd, mem_value);
            }
            35 => {
                let rs1 = fields::rs1(instr);
                let rs1_value = self.registers.get(rs1) as i64;
                let rs2 = fields::rs2(instr);
                let rs2_value = self.registers.get(rs2);
                let imm = fields::imm_stype(instr);
                let addr = rs1_value.wrapping_add(imm as i64) as usize;
                println!("sd *(x{rs1} + {imm}) = x{rs2}");
                self.data.write_doubleword(addr, rs2_value);
            }
            51 => {
                let rd = fields::rd(instr);
                let rs1 = fields::rs1(instr);
                let rs2 = fields::rs2(instr);
                let rs1_value = self.registers.get(rs1);
                let rs2_value = self.registers.get(rs2);
                match fields::funct3(instr) {
                    0 => match fields::funct7(instr) {
                        0 => {
                            println!("add, x{rd} = x{rs1} + x{rs2}");
                            self.registers.set(rd, rs1_value.wrapping_add(rs2_value))
                        }
                        32 => {
                            println!("sub, x{rd} = x{rs1} - x{rs2}");
                            self.registers.set(rd, rs1_value.wrapping_sub(rs2_value))
                        }
                        _ => unimplemented!("Expected funct7 = 0 or 32 for add/sub"),
                    },
                    6 => {
                        println!("and, x{rd} = x{rs1} & x{rs2}");
                        self.registers.set(rd, rs1_value & rs2_value)
                    }
                    7 => {
                        println!("or, x{rd} = x{rs1} | x{rs2}");
                        self.registers.set(rd, rs1_value | rs2_value)
                    }
                    _ => unimplemented!("Missing implementation for {instr}"),
                }
            }
            99 => {
                println!("beq -- not doing anything yet")
            },
	    19 => {
		let rd = fields::rd(instr);
		let rs1 = fields::rs1(instr);
		let rs1_value = self.registers.get(rs1);
		let imm = fields::imm_itype(instr);
		match fields::funct3(instr) {
		    0 => {
			println!("addi, x{rd} = x{rs1} + {imm}");
			// Need immediate as unsigned to do the register addition. First let
			// rust convert i16 to i64, which will sign extended, then just pretend
			// it is u64.
			let imm_sign_extended_unsigned = unsafe { mem::transmute(imm as i64) };
			self.registers.set(rd, rs1_value.wrapping_add(imm_sign_extended_unsigned));
		    },
		    _ => unimplemented!("Missing implementation for funct3 {} of {instr}", fields::funct3(instr)),
		}
	    },
	    103 => {
		let rd = fields::rd(instr);
		self.registers.set(rd, self.pc + 4);
		let rs1 = fields::rs1(instr);
		let rs1_value = self.registers.get(rs1);
		let imm = fields::imm_itype(instr);
		let imm_sign_extended_unsigned = unsafe { mem::transmute(imm as i64) };
		let rs1_value_plus_imm = rs1_value.wrapping_add(imm_sign_extended_unsigned);
		let target_address = 0xfffffffffffffff7 & rs1_value_plus_imm;
		self.pc = target_address;
		println!("jalr, x{rd} = pc + 4, pc = x{rs1} + {imm} = {target_address}");
		// Return to avoid incrementing program counter
		return Ok(());
		
	    },
            _ => unimplemented!("Missing implementation for opcode {op} of {instr:x}"),
        }

        // Increment program counter
        self.pc += 4;
	Ok(())
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU pc={:x}", self.pc);
        for n in 0..32 {
            let value = self.registers.get(n);
            if value != 0 {
                write!(f, " x{n}={}", self.registers.get(n))?;
            }
        }
        writeln!(f, "\naddr.   instr.      data")?;
        for addr in (0..32).step_by(4) {
            let instruction = self.instructions.read_word(addr);
            let data = self.data.read_word(addr);
            writeln!(f, " {addr:02x}    {instruction:08x}   {data:08x}")?;
        }
        Ok(())
    }
}
