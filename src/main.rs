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

    // Low instruction word reserved for an illegal instruction (e.g.
    // for returns corresponding to no call)
    let text_load_offset = 8;
    
    write_constant_vector(&mut cpu, 1, 8, 0, 4 * 8);
    load_text_section(&mut cpu, &asm_file, text_load_offset);
    
    let (sym_offset, sym_size) = find_function_symbol(&asm_file, &symbol).expect("Symbol to be found");
    println!("Function {symbol} found at {sym_offset} with size {sym_size}");

    // Set the program counter to the function
    cpu.set_program_counter((text_load_offset + sym_offset).try_into().unwrap());
    println!("{cpu}");

    println!("PROGRAM EXECUTION STARTS HERE ------------------");
    
    for _ in (0..sym_size+1).step_by(4) {
        if let Err(err) =  cpu.execute_instruction() {
	    println!("Error: {err}. Stopping.")
	}
        println!("{cpu}");
    }

    println!("END --------------------------------------------");
}
