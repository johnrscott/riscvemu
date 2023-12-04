use clap::Parser;
use clap_num::maybe_hex;
use riscvemu::platform::eei::Eei;
use riscvemu::{elf_utils::load_elf, platform::Platform};
use rustyline::DefaultEditor;
use rustyline::error::ReadlineError;
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

    /// If an exception is encountered, print the exception along with
    /// the program counter and mcycle of the instruction causing the
    /// exception
    #[arg(short, long)]
    exceptions_are_errors: bool,

    /// Break on program counter match and begin debug stepping (use
    /// 0x prefix for hexadecimal)
    #[arg(short, long, value_parser=maybe_hex::<u32>)]
    pc_breakpoint: Option<u32>,

    /// Break on mcycle match and begin debug stepping (use 0x prefix
    /// for hexadecimal)
    #[arg(short, long, value_parser=maybe_hex::<u64>)]
    cycle_breakpoint: Option<u64>,
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

    // `()` can be used when no completer is required
    let mut rl = DefaultEditor::new().unwrap();
    
    loop {
        let readline = rl.readline("(db) ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str());
                println!("Line: {}", line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break
            }
        }
    }
    
    /*
    
    if args.debug
        || args.pc_breakpoint.is_some()
        || args.cycle_breakpoint.is_some()
    {
        let mut platform = Platform::new();
        platform.set_exceptions_are_errors(args.exceptions_are_errors);

        // Open an executable file
        let elf_name = args.input.to_string();
        load_elf(&mut platform, &elf_name).unwrap();

        if args.debug {
            platform.set_trace(true);
            loop {

                if let Err(ex) = platform.step() {
                    println!(
                        "Got exception {ex:?} at pc=0x{:x}, mcycle={}",
                        platform.pc(),
                        platform.mcycle()
                    );
		    return;
		}
		
		press_enter_to_continue();
            }
        } else {
            let mut step = false;
            loop {
                if let Some(pc_breakpoint) = args.pc_breakpoint {
                    if platform.pc() == pc_breakpoint {
                        platform.set_trace(true);
                        step = true;
                    }
                }

                if let Some(cycle_breakpoint) = args.cycle_breakpoint {
                    if platform.mcycle() == cycle_breakpoint {
                        platform.set_trace(true);
                        step = true;
                    }
                }

                if let Err(ex) = platform.step() {
                    println!(
                        "Got exception {ex:?} at pc=0x{:x}, mcycle={}",
                        platform.pc(),
                        platform.mcycle()
                    );
		    return;
		}

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
            platform.set_exceptions_are_errors(args.exceptions_are_errors);

            // Open an executable file
            let elf_name = args.input.to_string();

            if let Err(e) = load_elf(&mut platform, &elf_name) {
                println!("Error loading elf: {e}");
                return;
            }

            println!("Beginning execution\n");
            loop {

		if let Err(ex) = platform.step() {
                    println!(
                        "Got exception {ex:?} at pc=0x{:x}, mcycle={}",
                        platform.pc(),
                        platform.mcycle()
                    );
		    return;
		}

		
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
*/
}
