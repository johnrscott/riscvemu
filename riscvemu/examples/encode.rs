//! Instruction encoding and decoding
//!
//! Example showing how to use the instruction macros. See
//! The documentation for Instr for what each instruction does
//! in the different instruction subsets (e.g. RV32I and RV64I).

//use riscvemu::instr::decode::Rv32i;
use riscvemu::encode::*;

fn main() -> Result<(), &'static str> {
    // The constant -10 is the value that is loaded
    // into x1[31:12].
    let x = lui!(x1, -10);
    //let y = Rv32i::from(x);
    println!("{x}");

    // The constant 12 is bits [31:12] of the number
    // to be added to the pc
    let x = auipc!(x10, 12);
    //let y = Rv32i::from(x);
    println!("{x}");

    // The offset -26 must be even, and must fit in 21
    // bits.
    let x = jal!(x9, -26);
    //let y = Rv32i::from(x);
    println!("{x}");

    // The offset 23 can be odd; when the result is added
    // to x23, the low bit is zeroed before adding to the pc
    let x = jalr!(x2, x23, 23);
    //let y = Rv32i::from(x);
    println!("{x}");

    // Note that the offset 22 must be a multiple of 2
    let x = beq!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = bne!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = blt!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = bge!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = bltu!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = bltu!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = bgeu!(x2, x23, 22);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = lb!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = lh!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = lw!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = lbu!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = lhu!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = lwu!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = ld!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sb!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sh!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sw!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sd!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = addi!(x2, x1, -3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = slti!(x0, x0, 1);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sltiu!(x0, x0, 1);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = andi!(x2, x10, -20);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = ori!(x2, x10, -20);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = xori!(x2, x10, -20);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = addiw!(x2, x10, -20);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = slliw!(x2, x10, 3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = srliw!(x2, x10, 3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sraiw!(x2, x10, 3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = add!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sub!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sll!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = slt!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sltu!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = xor!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = srl!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sra!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = or!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = and!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = addw!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = subw!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sllw!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = srlw!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    let x = sraw!(x2, x10, x3);
    //let y = Rv32i::from(x);
    println!("{x}");

    Ok(())
}
