use std::fmt;

mod cpu;
mod elf_utils;
mod fields;
mod memory;
mod memory_patterns;
mod register_file;

use elf_utils::load_text_section;

use crate::{cpu::Cpu, elf_utils::read_all_symbols, memory_patterns::write_constant_vector};

fn main() {
    let mut cpu = Cpu::new();

    let asm_file = format!("asm/add_memory.out");

    write_constant_vector(&mut cpu, 1, 8, 0, 4 * 8);
    load_text_section(&mut cpu, &asm_file);

    read_all_symbols(&asm_file);
return;
    println!("{cpu}");
    for _ in 0..5 {
        cpu.execute_instruction();
        println!("{cpu}");
    }
}
