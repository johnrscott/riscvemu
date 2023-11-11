//! Execution of RISC-V instructions
//!
//! This file contains the functions which execute RISC-V
//! instructions. Each function takes the non-opcode data from the
//! instruction in a particular format (e.g. R-type), and a reference
//! to a hart on which the instruction is executing. Behaviour of the
//! instructions depends on both the instruction and the XLEN of the
//! base instruction format.
//!
//! Instruction behaviour is defined in RISC-V unprivileged
//! specification version 20191213

use crate::hart::{next_instruction_address, sign_extend, ExecutionError, Hart};

use super::{
    instr_type::{decode_btype, decode_itype, decode_jtype, decode_utype, Itype, SBtype, UJtype},
    rv32i::Branch,
};

use super::fields::*;
use std::mem;

/// Load upper immediate in 32-bit mode
///
/// Load the u_immediate into the upper 12 bits of the register
/// dest and fill the lower 20 bits with zeros. Set pc = pc + 4.
///
pub fn execute_lui_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let UJtype {
        rd: dest,
        imm: u_immediate,
    } = decode_utype(instr);
    hart.set_x(dest, u_immediate << 12)?;
    hart.increment_pc();
    Ok(())
}

/// Add upper immediate to program counter in 32-bit mode
///
/// Make a 32-bit value by setting its upper 12 bits to
/// u_immediate and its lower 20 bits to zero, and add
/// the current value of the program counter. Store the
/// result in the register dest. Set pc = pc + 4.
///
pub fn execute_auipc_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let UJtype {
        rd: dest,
        imm: u_immediate,
    } = decode_utype(instr);
    let value = hart.pc.wrapping_add(u_immediate << 12);
    hart.set_x(dest, value).unwrap();
    hart.increment_pc();
    Ok(())
}

/// Jump and link in 32-bit mode
///
/// Store the address of the next instruction (pc + 4) in
/// the register dest. Then set pc = pc + offset (an
/// unconditional jump relative to the program counter).
pub fn execute_jal_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let UJtype {
        rd: dest,
        imm: offset,
    } = decode_jtype(instr);
    let return_address = next_instruction_address(hart.pc);
    hart.set_x(dest, return_address)?;
    let relative_address = sign_extend(offset, 20);
    hart.jump_relative_to_pc(relative_address)
}

/// Jump and link register in 32-bit mode
///
/// Store the address of the next instruction (pc + 4) in
/// the register dest. Then compute base + offset, set the
/// least significant bit to zero, and set the pc to the
/// result.
pub fn execute_jalr_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let return_address = next_instruction_address(hart.pc);
    hart.set_x(dest, return_address)?;
    let relative_address = sign_extend(offset, 11);
    let base_address = hart.x(base)?;
    let new_pc = 0xffff_fffe & base_address.wrapping_add(relative_address);
    hart.jump_to_address(new_pc)
}

/// Execute a conditional branch in 32-bit mode
///
/// Compute a condition specified by the mnemonic between the values
/// of the registers src1 and src2. If the result is false, do nothing.
/// Else, compute a pc-relative address by sign-extending offset, and
/// jump to that address, raising an address-misaligned exception if the
/// resulting program counter is not aligned to a 4-byte boundary.
pub fn execute_branch_rv32i(
    hart: &mut Hart,
    mnemonic: Branch,
    instr: u32,
) -> Result<(), ExecutionError> {
    let SBtype {
        rs1: src1,
        rs2: src2,
        imm: offset,
    } = decode_btype(instr);
    let src1 = hart.x(src1)?;
    let src2 = hart.x(src2)?;
    let branch_taken = match mnemonic {
        Branch::Beq => src1 == src2,
        Branch::Bne => src1 != src2,
        Branch::Blt => {
            let src1: i32 = interpret_u32_as_signed!(src1);
            let src2: i32 = interpret_u32_as_signed!(src2);
            src1 < src2
        }
        Branch::Bge => {
            let src1: i32 = interpret_u32_as_signed!(src1);
            let src2: i32 = interpret_u32_as_signed!(src2);
            src1 >= src2
        }
        Branch::Bltu => src1 < src2,
        Branch::Bgeu => src1 >= src2,
    };

    if branch_taken {
        let relative_address = sign_extend(offset, 11);
        hart.jump_relative_to_pc(relative_address)
    } else {
        hart.increment_pc();
        Ok(())
    }
}
