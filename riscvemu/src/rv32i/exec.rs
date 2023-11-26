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

use crate::{
    hart::{memory::Wordsize, next_instruction_address, ExecutionError, Hart},
    instr_type::{decode_rtype, decode_stype, Rtype},
};

use crate::instr_type::{
    decode_btype, decode_itype, decode_jtype, decode_utype, Itype, SBtype, UJtype,
};

use crate::fields::{interpret_i32_as_unsigned, interpret_u32_as_signed, sign_extend};

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

fn get_branch_data(hart: &Hart, instr: u32) -> Result<(u32, u32, u16), ExecutionError> {
    let SBtype {
        rs1: src1,
        rs2: src2,
        imm: offset,
    } = decode_btype(instr);
    let src1 = hart.x(src1)?;
    let src2 = hart.x(src2)?;
    Ok((src1, src2, offset))
}
fn do_branch(hart: &mut Hart, branch_taken: bool, offset: u16) -> Result<(), ExecutionError> {
    if branch_taken {
        let relative_address = sign_extend(offset, 11);
        hart.jump_relative_to_pc(relative_address)
    } else {
        hart.increment_pc();
        Ok(())
    }
}

pub fn execute_beq_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, offset) = get_branch_data(hart, instr)?;
    let branch_taken = src1 == src2;
    do_branch(hart, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bne_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, offset) = get_branch_data(hart, instr)?;
    let branch_taken = src1 != src2;
    do_branch(hart, branch_taken, offset)?;
    Ok(())
}

pub fn execute_blt_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, offset) = get_branch_data(hart, instr)?;
    let branch_taken = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1 < src2
    };
    do_branch(hart, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bge_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, offset) = get_branch_data(hart, instr)?;
    let branch_taken = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1 >= src2
    };
    do_branch(hart, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bltu_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, offset) = get_branch_data(hart, instr)?;
    let branch_taken = src1 < src2;
    do_branch(hart, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bgeu_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, offset) = get_branch_data(hart, instr)?;
    let branch_taken = src1 >= src2;
    do_branch(hart, branch_taken, offset)?;
    Ok(())
}

pub fn execute_lb_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = sign_extend(
        u32::try_from(
            hart.memory
                .read(load_address.into(), Wordsize::Byte)
                .unwrap(),
        )
        .unwrap(),
        7,
    );
    hart.set_x(dest, load_data)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_lh_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = sign_extend(
        u32::try_from(
            hart.memory
                .read(load_address.into(), Wordsize::Halfword)
                .unwrap(),
        )
        .unwrap(),
        15,
    );
    hart.set_x(dest, load_data)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_lw_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = hart
        .memory
        .read(load_address.into(), Wordsize::Word)
        .unwrap()
        .try_into()
        .unwrap();
    hart.set_x(dest, load_data)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_lbu_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = hart
        .memory
        .read(load_address.into(), Wordsize::Byte)
        .unwrap()
        .try_into()
        .unwrap();
    hart.set_x(dest, load_data)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_lhu_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = hart
        .memory
        .read(load_address.into(), Wordsize::Halfword)
        .unwrap()
        .try_into()
        .unwrap();
    hart.set_x(dest, load_data)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_sb_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let SBtype {
        rs1: base,
        rs2: src,
        imm: offset,
    } = decode_stype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = hart.x(src)?;
    hart.memory
        .write(store_address.into(), store_data.into(), Wordsize::Byte)
        .unwrap();
    hart.increment_pc();
    Ok(())
}

pub fn execute_sh_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let SBtype {
        rs1: base,
        rs2: src,
        imm: offset,
    } = decode_stype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = hart.x(src)?;
    hart.memory
        .write(store_address.into(), store_data.into(), Wordsize::Halfword)
        .unwrap();
    hart.increment_pc();
    Ok(())
}

pub fn execute_sw_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let SBtype {
        rs1: base,
        rs2: src,
        imm: offset,
    } = decode_stype(instr);
    let base_address = hart.x(base)?;
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = hart.x(src)?;
    hart.memory
        .write(store_address.into(), store_data.into(), Wordsize::Word)
        .unwrap();
    hart.increment_pc();
    Ok(())
}

fn reg_imm_values(hart: &Hart, instr: u32) -> Result<(u32, u8, u32), ExecutionError> {
    let Itype {
        rs1: src,
        imm: i_immediate,
        rd: dest,
    } = decode_itype(instr);
    let src: u32 = hart.x(src)?;
    let i_immediate = sign_extend(i_immediate, 11);
    Ok((src, dest, i_immediate))
}

pub fn execute_addi_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = src.wrapping_add(i_immediate);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_slti_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = {
        let src: i32 = interpret_u32_as_signed!(src);
        let i_immediate: i32 = interpret_u32_as_signed!(i_immediate);
        (src < i_immediate) as u32
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_sltiu_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = (src < i_immediate) as u32;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_andi_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = src & i_immediate;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_ori_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = src | i_immediate;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_xori_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = src ^ i_immediate;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_slli_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = src << (0x1f & i_immediate);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_srli_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = src >> (0x1f & i_immediate);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_srai_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src, dest, i_immediate) = reg_imm_values(hart, instr)?;
    let value = {
        let src: i32 = interpret_u32_as_signed!(src);
        interpret_i32_as_unsigned!(src >> (0x1f & i_immediate))
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

fn reg_reg_values(hart: &Hart, instr: u32) -> Result<(u32, u32, u8), ExecutionError> {
    let Rtype {
        rs1: src1,
        rs2: src2,
        rd: dest,
    } = decode_rtype(instr);
    let src1: u32 = hart.x(src1)?;
    let src2: u32 = hart.x(src2)?;
    Ok((src1, src2, dest))
}

pub fn execute_add_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1.wrapping_add(src2);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_sub_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1.wrapping_sub(src2);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_slt_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        (src1 < src2) as u32
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_sltu_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = (src1 < src2) as u32;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_and_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1 & src2;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_or_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1 | src2;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_xor_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1 ^ src2;
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_sll_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1 << (0x1f & src2);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_srl_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1 >> (0x1f & src2);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_sra_rv32i(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        interpret_i32_as_unsigned!(src1 >> (0x1f & src2))
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}
