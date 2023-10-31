use riscvemu::{hart::Hart, elf_utils::load_text_section};

fn main() {
    
    let mut hart = Hart::default();
    let binary = format!("c/hello.out");

    // Load text section at 0 offset
    load_text_section(&mut hart, &binary, 0);

    for _ in 0..10000 {
	if let Err(e) = hart.step() {
	    println!("Trap: {e} at instruction pc={:x}", hart.pc);
	    break;
	}
    }
}
