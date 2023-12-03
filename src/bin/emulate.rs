use clap::Parser;
use clap_num::maybe_hex;
use riscvemu::platform::eei::Eei;
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

    /// Break on program counter match and begin debug stepping (use
    /// 0x prefix for hexadecimal)
    #[arg(short, long, value_parser=maybe_hex::<u32>)]
    breakpoint: Option<u32>,
    
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
    
    if args.debug || args.breakpoint.is_some() {

        let mut platform = Platform::new();

        // Open an executable file
        let elf_name = args.input.to_string();
        load_elf(&mut platform, &elf_name).unwrap();
	
	if args.debug {
            platform.set_trace(true);
            loop {
		platform.step();
		press_enter_to_continue();
            }
	} else {
	    let mut step = false;
            loop {
		if platform.pc() == args.breakpoint.unwrap() {
		    platform.set_trace(true);
		    step = true;
		}
		platform.step();
		if step {
		    press_enter_to_continue();
		}
            }	    
	}
	
    } else {
	
	let (uart_tx, uart_rx) = mpsc::channel();

	// Thread running the emulation
	let emulator_handle = thread::spawn(move || {
            let mut platform = Platform::new();
            //platform.set_trace(true);

            // Open an executable file
            let elf_name = args.input.to_string();
	    
            if let Err(e) = load_elf(&mut platform, &elf_name) {
		println!("Error loading elf: {e}");
		return;
	    }

            println!("Beginning execution\n");
            loop {
		platform.step();
		//press_enter_to_continue();
		uart_tx.send(platform.flush_uartout()).unwrap();
            }
	});

	// Thread for printing received UART stdout
	let uart_host_handle = thread::spawn(move || loop {
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
