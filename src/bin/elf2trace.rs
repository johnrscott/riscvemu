use riscvemu::trace_file::elf_to_trace_file;
use clap::Parser;

/// Program to convert an ELF executable file to a trace image file
///
/// This file converts executable files compiled using the
/// riscv-gcc-toolchain to a simple human-readable instruction-region
/// image format used for creating testbench traces. The format stores
/// the contents of the EEPROM region (instructions and read-only
/// memory).
///
/// The format of the trace file is as follows. Excess white space is
/// ignored, and any remaining part of a line starting from # is a
/// comment and is treated as white space. The file contains sections,
/// which are indicated by a label that begins with a dot
/// (i.e. .section).
///
/// The .eeprom section contains the memory image of the EEPROM
/// region, which contains read-only data and instructions. Lines in
/// this section have the format:
///
/// ADDR INSTR # optional comment
///
/// Addresses where the memory is zero can be omitted. All numbers in
/// the file are in hexadecimal, and are not prefixed by 0x. Numbers
/// may be padded to 8 characters.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about)]
struct Args {
    /// Path to input ELF file 
    #[arg(short, long)]
    input: String,

    /// Path to output file
    #[arg(short, long)]
    output: String,
}

fn main() {
    let args = Args::parse();
    match elf_to_trace_file(args.input.clone(), args.output) {
	Err(e) => println!("{e}"),
	Ok(_) => ()
    }
}
