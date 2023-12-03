use clap::Parser;
use riscvemu::trace_file::elf_to_trace_file;

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
/// the .eeprom section are in hexadecimal, and are not prefixed by
/// 0x. Numbers may be padded to 8 characters.
///
/// The other sections in the file are trace points, which describe
/// the state of the core at a particular cycle. The sections are
/// called .trace.<cycle>, where <cycle> is the value of the 64-bit
/// mcycle register. The execution model assumes mcycle is incremented
/// after each instruction is executed; for example, mcycle=0
/// corresponds to the reset state of the processor (after no
/// instructions have been retired), and mcycle=1 is the state after
/// one instruction has executed.
///
/// Lines in the .trace.* section have the format:
///
/// KEY VALUE # optional comment
///
/// The key names a property of the processor, without quotes (for
/// example x3 represents a register, pc means the program counter,
/// and uart means the characters received over the debug UART). The
/// value is the expected state of the property. Strings are contained
/// in quotes (for example, uart "Hello World"), and integers are
/// decimal by default (and may be signed), or hexademical (and
/// unsigned) if they are prefixed by 0x.
///
/// Strings may contains standard ASCII escape characters. The list
/// of supported escape sequences is as follows:
/// * \n: newline character
///
/// Trace points do not need to be listed in cycle order, but they
/// will be checked in cycle order during emulation.
///
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
        Ok(_) => (),
    }
}
