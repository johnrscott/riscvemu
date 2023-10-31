use riscvemu::hart::memory::Wordsize;
use riscvemu::instr::decode::Instr;
use riscvemu::{hart::Hart, elf_utils::load_elf};
use std::io;
use std::io::prelude::*;

fn press_enter_to_continue() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press enter to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn reg_index_to_abi_name(index: usize) -> String {
    String::from(match index {
	0 => "zero",
	1 => "ra (return address)",
	2 => "sp (stack pointer)",
	3 => "gp (global pointer)",
	4 => "tp (thread_pointer)",
	5 => "t0 (temp 0)",
	6 => "t1 (temp 1)",
	7 => "t2 (temp 2)",
	8 => "s0/fp (saved register/frame pointer)",
	9 => "s1 (saved register)",
	10 => "a0 (function args/return values)",
	11 => "a1 (function args/return values)",
	12 => "a2 (function args)",
	13 => "a3 (function args)",
	14 => "a4 (function args)",
	15 => "a5 (function args)",
	16 => "a6 (function args)",
	17 => "a7 (function args)",
	_ => unimplemented!("Not implemented this register name yet")
    })
}

fn print_nonzero_registers(hart: &Hart) {
    for n in 0..32 {
	let value = hart.registers.read(n).unwrap();
	if value != 0 {
	    let reg_abi_name = reg_index_to_abi_name(n);
	    println!("{reg_abi_name}: 0x{value:x}");
	}
    }
}
  

fn main() {
    
    let mut hart = Hart::default();
    let elf_name = format!("c/hello.out");

    // Load text section at 0 offset
    load_elf(&mut hart, &elf_name);

    let debug = false;

    println!("Starting execution");
    
    for _ in 0..10000 {
	let pc = hart.pc;
	let instr = hart.memory.read(pc.into(), Wordsize::Word).unwrap();
	let instr = Instr::from(instr.try_into().unwrap());

	if debug {
	    println!("\nCurrent state of hart:");
	    //println!("{:x?}\n", hart);
	    print_nonzero_registers(&hart);
	    println!("\nCurrent pc = 0x{pc:x}");
	    println!("Next instruction: {:x?}", instr);
	    press_enter_to_continue();
	    print!("Executing instruction now... ");

	}
	if let Err(e) = hart.step() {
	    println!("trap: {e} at instruction pc={:x}", hart.pc);
	    break;
	} else {
	    if debug {
		println!("done (no trap)");
	    }
	}
    }
}
