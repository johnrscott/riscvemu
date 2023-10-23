use std::fmt;

mod cpu;
mod elf_utils;
mod fields;
mod memory;
mod memory_patterns;
mod register_file;

use elf_utils::load_text_section;

use crate::{cpu::Cpu, elf_utils::find_function_symbol, memory_patterns::write_constant_vector};

fn main() {
    let mut cpu = Cpu::new();

    let asm_file = format!("asm/add_memory.out");
    let symbol = format!("set_memory");

    write_constant_vector(&mut cpu, 1, 8, 0, 4 * 8);
    load_text_section(&mut cpu, &asm_file);
    
    let (sym_offset, sym_size) = find_function_symbol(&asm_file, &symbol).expect("Symbol to be found");
    println!("Function {symbol} found at {sym_offset} with size {sym_size}");

    // Set the program counter to the function
    cpu.set_program_counter(sym_offset.try_into().unwrap());
    println!("{cpu}");
    
    for _ in 0..sym_size {
        cpu.execute_instruction();
        println!("{cpu}");
    }
}
