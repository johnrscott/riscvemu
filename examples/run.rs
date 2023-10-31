use riscvemu::hart::memory::Wordsize;
use riscvemu::instr::decode::Instr;
use riscvemu::{hart::Hart, elf_utils::load_text_section};
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

fn main() {
    
    let mut hart = Hart::default();
    let binary = format!("c/hello.out");

    // Load text section at 0 offset
    load_text_section(&mut hart, &binary, 0);

    let debug = true;
    
    for _ in 0..10000 {
	let pc = hart.pc;
	println!("Current pc = 0x{pc:x}");
	let instr = hart.memory.read(pc.into(), Wordsize::Word).unwrap();
	let instr = Instr::from(instr.try_into().unwrap());
	println!("Next instruction: {:x?}", instr);
	print!("Executing instruction now... ");
	if let Err(e) = hart.step() {
	    println!("trap: {e} at instruction pc={:x}", hart.pc);
	    break;
	} else {
	    println!("done (no trap)");
	}
	println!("Current state of hart:");
	println!("{:x?}", hart);
	press_enter_to_continue();
    }
}
