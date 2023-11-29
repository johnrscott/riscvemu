use riscvemu::{elf_utils::load_elf, hart::platform::Platform};
use std::sync::mpsc;
use std::{io, thread};
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

    let (tx, rx) = mpsc::channel();

    // Thread running the emulation
    let ucontroller_handle = thread::spawn(move || {

	let mut platform = Platform::new();
	//platform.set_trace(true);

	// Open an executable file
	let elf_name = "../c/hello.out".to_string();
	load_elf(&mut platform, &elf_name);

	println!("Beginning execution\n");
	loop {
            platform.step_clock();
            //press_enter_to_continue();
	    tx.send(platform.flush_uartout()).unwrap();
	}
    });

    // Thread for printing received UART stdout
    let uart_host_handle = thread::spawn(move||{
	loop {
	    if let Ok(uart_rx) = rx.recv() {
		print!("{uart_rx}");
	    } else {
		println!("UART channel closed");
		break;
	    }
	}
    });

    uart_host_handle.join().unwrap();
    ucontroller_handle.join().unwrap();
}
    
