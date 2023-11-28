use riscvemu::{elf_utils::load_elf, hart::platform::Platform};
use std::io;
use std::io::{Read, Write};

fn press_enter_to_continue() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press enter to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {
    let mut platform = Platform::new();
    platform.set_trace(true);

    // Open an executable file
    let elf_name = "../c/hello.out".to_string();
    load_elf(&mut platform, &elf_name);

    println!("Beginning execution\n");
    loop {
	platform.step_clock();
	press_enter_to_continue();
    }
}
