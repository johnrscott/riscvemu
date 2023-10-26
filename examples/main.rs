//! Instruction encoding and decoding
//!
//! Example showing how to use the instruction macros. See
//! The documentation for Instr for what each instruction does
//! in the different instruction subsets (e.g. RV32I and RV64I).

use riscvemu::instr_decode::Instr;
use riscvemu::instr_encode::*;
use riscvemu::instr_opcodes::*;

fn main() -> Result<(), &'static str> {
    // The constant -10 is the value that is loaded
    // into x1[31:12].
    let x = lui!(x1, -10);
    let y = Instr::from(x);
    println!("{y}");

    // The constant 12 is bits [31:12] of the number
    // to be added to the pc
    let x = auipc!(x10, 12);
    let y = Instr::from(x);
    println!("{y}");

    // The offset -26 must be even, and must fit in 21
    // bits.
    let x = jal!(x9, -26);
    let y = Instr::from(x);
    println!("{y}");

    // The offset 23 can be odd; when the result is added
    // to x23, the low bit is zeroed before adding to the pc
    let x = jalr!(x2, x23, 23);
    let y = Instr::from(x);
    println!("{y}");

    // Note that the offset 22 must be a multiple of 2
    let x = beq!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = bne!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = blt!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = bge!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = bltu!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = bltu!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = bgeu!(x2, x23, 22);
    let y = Instr::from(x);
    println!("{y}");

    let x = lb!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = lh!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");
    
    let x = lw!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");
    
    let x = lbu!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = lhu!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = lwu!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = ld!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = sb!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");
    
    let x = sh!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");
    
    let x = sw!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = sd!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = addi!(x2, x1, -3);
    let y = Instr::from(x);
    println!("{y}");

    let x = slti!(x0, x0, 1);
    let y = Instr::from(x);
    println!("{y}");

    let x = sltiu!(x0, x0, 1);
    let y = Instr::from(x);
    println!("{y}");

    let x = andi!(x2, x10, -20);
    let y = Instr::from(x);
    println!("{y}");

    let x = ori!(x2, x10, -20);
    let y = Instr::from(x);
    println!("{y}");

    let x = xori!(x2, x10, -20);
    let y = Instr::from(x);
    println!("{y}");

    let x = add!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = sub!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = sll!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");
    
    let x = slt!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = sltu!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = xor!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = srl!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = sra!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    let x = or!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");
    
    let x = and!(x2, x10, x3);
    let y = Instr::from(x);
    println!("{y}");

    Ok(())
}
