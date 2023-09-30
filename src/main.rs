use std::collections::HashMap;
use std::fmt;
use std::mem;

fn n_bit_mask(num_bits: u64) -> u64 {
    (1 << num_bits) - 1
}

fn extract_bit_range(instr: u64, start: u64, width: u64) -> u64 {
    let end = start + width - 1;
    if end >= 64 {
        panic!("This field [{end}:{start}] does not fall within a u64");
    }
    n_bit_mask(width) & (instr >> start)
}

fn opcode(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 0, 7) as u8
}

fn rd(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 7, 5) as u8
}

fn funct3(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 12, 3) as u8
}

fn rs1(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 15, 5) as u8
}

fn rs2(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 20, 5) as u8
}

fn funct7(instr: u32) -> u8 {
    extract_bit_range(instr.into(), 25, 7) as u8
}

fn imm_itype(instr: u32) -> i16 {
    let mut unsigned = extract_bit_range(instr.into(), 20, 12) as u16;
    let sign_bit = 1 & (unsigned >> 11);
    if sign_bit == 1 {
        unsigned = 0xf000u16 | unsigned;
    }
    unsafe { mem::transmute(unsigned) }
}

fn imm_stype(instr: u32) -> i16 {
    let imm11_5 = funct7(instr) as u16;
    let imm4_0 = rd(instr) as u16;
    let mut unsigned = (imm11_5 << 4) | imm4_0;
    let sign_bit = 1 & (unsigned >> 11);
    if sign_bit == 1 {
        unsigned = 0xf000u16 | unsigned;
    }
    unsafe { mem::transmute(unsigned) }
}




#[derive(Debug)]
struct Memory {
    data: HashMap<usize, u8>,
}

impl Memory {
    fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    fn write_byte(&mut self, addr: usize, value: u8) {
        self.data.insert(addr, value);
    }

    fn read_byte(&self, addr: usize) -> u8 {
        if let Some(value) = self.data.get(&addr) {
            *value
        } else {
            0
        }
    }

    fn write_halfword(&mut self, addr: usize, value: u16) {
        self.write_byte(addr, extract_bit_range(value.into(), 0, 8) as u8);
        self.write_byte(addr + 1, extract_bit_range(value.into(), 8, 8) as u8)
    }

    fn read_halfword(&self, addr: usize) -> u16 {
        let byte_0 = self.read_byte(addr);
        let byte_1 = self.read_byte(addr + 1);
        u16::from_le_bytes([byte_0, byte_1])
    }

    fn write_word(&mut self, addr: usize, value: u32) {
        self.write_halfword(addr, extract_bit_range(value.into(), 0, 16) as u16);
        self.write_halfword(addr + 2, extract_bit_range(value.into(), 16, 16) as u16)
    }

    fn read_word(&self, addr: usize) -> u32 {
        let halfword_0 = self.read_halfword(addr);
        let halfword_1 = self.read_halfword(addr + 2);
        ((halfword_1 as u32) << 16) | halfword_0 as u32
    }

    fn write_doubleword(&mut self, addr: usize, value: u64) {
        self.write_word(addr, extract_bit_range(value.into(), 0, 32) as u32);
        self.write_word(addr + 4, extract_bit_range(value.into(), 32, 32) as u32)
    }

    fn read_doubleword(&self, addr: usize) -> u64 {
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

#[derive(Debug)]
struct RegisterFile {
    registers: [u64; 32],
}

impl RegisterFile {
    fn new() -> Self {
        Self { registers: [0; 32] }
    }

    fn set(&mut self, which: u8, value: u64) {
        if which >= 32 {
            panic!("Invalid attempt to address register {which}")
        }
        if which != 0 {
            self.registers[which as usize] = value
        }
    }

    fn get(&self, which: u8) -> u64 {
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

#[derive(Debug)]
struct Cpu {
    registers: RegisterFile,
    pc: usize,
    instructions: Memory,
    data: Memory,
}

impl Cpu {
    fn new() -> Self {
        Self {
            registers: RegisterFile::new(),
            pc: 0,
            instructions: Memory::new(),
            data: Memory::new(),
        }
    }

    fn set_program_counter(&mut self, pc: usize) {
        if pc % 4 != 0 {
            panic!("Program counter {pc} is not a multiple of 4");
        }
        self.pc = pc;
    }

    fn write_instruction(&mut self, pc: usize, instr: u32) {
        if pc % 4 != 0 {
            panic!("Program counter {pc} is not a multiple of 4");
        }
        self.instructions.write_word(pc, instr);
    }

    fn write_data(&mut self, addr: usize, value: u8) {
        self.data.write_byte(addr, value);
    }

    fn execute_instruction(&mut self) {
        // Read the next instruction
        let instr = self.instructions.read_word(self.pc);

        // Check which instruction is being executed
        let op = opcode(instr);
        match op {
            3 => {
                let rd = rd(instr);
                let rs1 = rs1(instr);
                let rs1_value = self.registers.get(rs1) as i64;
                let imm = imm_itype(instr);
                let addr = rs1_value.wrapping_add(imm as i64) as usize;
                let mem_value = self.data.read_doubleword(addr);
                println!("ld x{rd} = *(x{rs1} + {imm})");
                self.registers.set(rd, mem_value);
            }
            35 => {
		let rs1 = rs1(instr);
		let rs1_value = self.registers.get(rs1) as i64;
		let rs2 = rs2(instr);
		let rs2_value = self.registers.get(rs2);
		let imm = imm_stype(instr);
		let addr = rs1_value.wrapping_add(imm as i64) as usize;
                println!("sd *(x{rs1} + {imm}) = x{rs2}");
		self.data.write_doubleword(addr, rs2_value);
            }
            51 => {
                let rd = rd(instr);
                let rs1 = rs1(instr);
                let rs2 = rs2(instr);
                let rs1_value = self.registers.get(rs1);
                let rs2_value = self.registers.get(rs2);
                match funct3(instr) {
                    0 => match funct7(instr) {
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
                println!("beq")
            }
            _ => unimplemented!("Missing implementation for opcode {op} of {instr}"),
        }

        // Increment program counter
        self.pc += 4;
    }
}

impl fmt::Display for Cpu {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CPU pc={}", self.pc);
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

fn main() {
    let mut cpu = Cpu::new();

    cpu.write_data(0 * 8, 10);
    cpu.write_data(1 * 8, 2);
    cpu.write_data(2 * 8, 3);
    cpu.write_data(3 * 8, 4);

    cpu.write_instruction(0, 0x00003083); // ld x1, 0(x0)
    cpu.write_instruction(4, 0xffe0b103); // ld x2, -2(x1)
    cpu.write_instruction(8, 0x00208233); // add x4, x1, x2
    cpu.write_instruction(12, 0x401201b3); // sub x3, x4, x1
    cpu.write_instruction(16, 0x001131a3); // sd x1, 3(x2)

    // cpu.write_instruction(8, 0x403100b3); // sub x1, x2, x3
    // cpu.write_instruction(12, 0x003170b3); // and x1, x2, x3
    // cpu.write_instruction(16, 0x003160b3); // or x1, x2, x3
    // cpu.write_instruction(20, 0x001131a3); // sd x1, 3(x2)
    // cpu.write_instruction(24, 0xfe310ee3); // beq x2, x3, -3

    println!("{cpu}");
    for _ in 0..5 {
        cpu.execute_instruction();
        println!("{cpu}");
    }
}
