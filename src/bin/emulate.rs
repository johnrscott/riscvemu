use clap::Parser;
use riscvemu::{elf_utils::load_elf, platform::Platform};
use std::io::{Read, Write};
use std::sync::mpsc;
use std::{io, thread};

/// Emulate a 32-bit RISC-V processor
///
/// 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Path to input executable file
    input: String,

    /// Single step through each instruction and print state
    #[arg(short, long)]
    debug: bool,

    /// The number of clock cycles to be emulated
    #[arg(short, long, default_value_t = 1_000_000)]
    cycles: usize,
}

fn press_enter_to_continue() {
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    write!(stdout, "Press enter to continue...").unwrap();
    stdout.flush().unwrap();

    // Read a single byte and discard
    let _ = stdin.read(&mut [0u8]).unwrap();
}

fn main() {

    let args = Args::parse();
    
    if args.debug {
        let mut platform = Platform::new();
        platform.set_trace(true);

        // Open an executable file
        let elf_name = args.input.to_string();
        load_elf(&mut platform, &elf_name).unwrap();

        println!("Beginning execution\n");
        loop {
	    platform.step();
	    press_enter_to_continue();
	    //uart_tx.send(platform.flush_uartout()).unwrap();
        }

    } else {
	
	let (uart_tx, uart_rx) = mpsc::channel();

	// Thread running the emulation
	let emulator_handle = thread::spawn(move || {
            let mut platform = Platform::new();
            //platform.set_trace(true);

            // Open an executable file
            let elf_name = args.input.to_string();
            load_elf(&mut platform, &elf_name).unwrap();

            println!("Beginning execution\n");
            loop {
		platform.step();
		//press_enter_to_continue();
		uart_tx.send(platform.flush_uartout()).unwrap();
            }
	});

	// Thread for printing received UART stdout
	let uart_host_handle = thread::spawn(move || loop {

	    // Read 

	    if let Ok(uart_rx) = uart_rx.recv() {
		print!("{uart_rx}");
            } else {
		println!("UART channel closed");
		break;
            }
	});

	uart_host_handle.join().unwrap();
	emulator_handle.join().unwrap();
    }
}
