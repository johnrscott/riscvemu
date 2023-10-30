use riscvemu::{hart::Hart, elf_utils::load_text_section};

fn main() {

    
    let mut hart = Hart::default();
    let binary = format!("c/hello.out");

    // Load text section at 0 offset
    load_text_section(&mut hart, &binary, 0);

    println!("{:x?}", hart);

    for n in 0..10 {
	match hart.step() {
            Ok(_) => println!("Done"),
        Err(trap) => println!("{trap}"),
	}
    }

    println!("{:x?}", hart);
}
