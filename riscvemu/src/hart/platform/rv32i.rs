use crate::{
    fields::sign_extend,
    hart::{machine::Exception, memory::Wordsize},
    instr_type::{decode_btype, decode_itype, decode_jtype, decode_utype, Itype, SBtype, UJtype, decode_stype, Rtype, decode_rtype}, interpret_u32_as_signed, interpret_i32_as_unsigned,
};

use super::eei::Eei;

fn check_instruction_address_aligned(pc: u32) -> Result<(), Exception> {
    if pc % 4 != 0 {
	Err(Exception::InstructionAddressMisaligned)
    } else {
	Ok(())
    }
}

fn jump_to_address<E: Eei>(eei: &mut E, target_pc: u32) -> Result<(), Exception> {
    check_instruction_address_aligned(target_pc)?;
    eei.set_pc(target_pc);
    Ok(())
}


/// Used to make the jump in conditional/unconditional branch
/// instructions, where a branch to an invalid. If the resulting
/// program counter would be invalid, then the program counter is
/// not modified, and an instruction address misaligned exception
/// is returned, as per section 2.5 of the unprivileged spec.
fn jump_relative_to_pc<E: Eei>(eei: &mut E, pc_relative_address: u32) -> Result<(), Exception> {
    let target_pc = eei.pc().wrapping_add(pc_relative_address);
    check_instruction_address_aligned(target_pc)?;
    eei.set_pc(target_pc);
    Ok(())
}

/// Load upper immediate in 32-bit mode
///
/// Load the u_immediate into the upper 12 bits of the register
/// dest and fill the lower 20 bits with zeros. Set pc = pc + 4.
///
pub fn execute_lui<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let UJtype {
        rd: dest,
        imm: u_immediate,
    } = decode_utype(instr);
    eei.set_x(dest, u_immediate << 12);
    eei.increment_pc();
    Ok(())
}

/// Add upper immediate to program counter in 32-bit mode
///
/// Make a 32-bit value by setting its upper 12 bits to
/// u_immediate and its lower 20 bits to zero, and add
/// the current value of the program counter. Store the
/// result in the register dest. Set pc = pc + 4.
///
pub fn execute_auipc<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let UJtype {
        rd: dest,
        imm: u_immediate,
    } = decode_utype(instr);
    let value = eei.pc().wrapping_add(u_immediate << 12);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

/// Jump and link in 32-bit mode
///
/// Store the address of the next instruction (pc + 4) in
/// the register dest. Then set pc = pc + offset (an
/// unconditional jump relative to the program counter).
///
/// This instruction will generate an instruction address
/// misaligned exception if the target program counter of
/// the jump would be misaligned. If this exception is
/// raised, then the dest register is not modified.
pub fn execute_jal<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let UJtype {
        rd: dest,
        imm: offset,
    } = decode_jtype(instr);
    let return_address = eei.pc().wrapping_add(4);
    let pc_relative_address = sign_extend(offset, 20);
    jump_relative_to_pc(eei, pc_relative_address)?;
    eei.set_x(dest, return_address);
    Ok(())
}

/// Jump and link register in 32-bit mode
///
/// Store the address of the next instruction (pc + 4) in
/// the register dest. Then compute base + offset, set the
/// least significant bit to zero, and set the pc to the
/// result.
///
/// This instruction will generate an instruction address
/// misaligned exception if the target program counter of
/// the jump would be misaligned. If this exception is
/// raised, then the dest register is not modified.
pub fn execute_jalr<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let return_address = eei.pc().wrapping_add(4);
    let relative_address = sign_extend(offset, 11);
    let base_address = eei.x(base);
    let target_pc = 0xffff_fffe & base_address.wrapping_add(relative_address);
    jump_to_address(eei, target_pc)?;
    eei.set_x(dest, return_address);
    Ok(())
}

fn get_branch_data<E: Eei>(eei: &E, instr: u32) -> (u32, u32, u16) {
    let SBtype {
        rs1: src1,
        rs2: src2,
        imm: offset,
    } = decode_btype(instr);
    let src1 = eei.x(src1);
    let src2 = eei.x(src2);
    (src1, src2, offset)
}

fn do_branch<E: Eei>(eei: &mut E, branch_taken: bool, offset: u16) -> Result<(), Exception> {
    if branch_taken {
        let pc_relative_address = sign_extend(offset, 11);
        jump_relative_to_pc(eei, pc_relative_address)?;
    } else {
        eei.increment_pc();
    }
    Ok(())
}

pub fn execute_beq<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, offset) = get_branch_data(eei, instr);
    let branch_taken = src1 == src2;
    do_branch(eei, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bne<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, offset) = get_branch_data(eei, instr);
    let branch_taken = src1 != src2;
    do_branch(eei, branch_taken, offset)?;
    Ok(())
}

pub fn execute_blt<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, offset) = get_branch_data(eei, instr);
    let branch_taken = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1 < src2
    };
    do_branch(eei, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bge<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, offset) = get_branch_data(eei, instr);
    let branch_taken = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1 >= src2
    };
    do_branch(eei, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bltu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, offset) = get_branch_data(eei, instr);
    let branch_taken = src1 < src2;
    do_branch(eei, branch_taken, offset)?;
    Ok(())
}

pub fn execute_bgeu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, offset) = get_branch_data(eei, instr);
    let branch_taken = src1 >= src2;
    do_branch(eei, branch_taken, offset)?;
    Ok(())
}

pub fn execute_lb<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = sign_extend(
        eei.load(load_address.into(), Wordsize::Byte)?,
        7,
    );
    eei.set_x(dest, load_data);
    eei.increment_pc();
    Ok(())
}

pub fn execute_lh<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = sign_extend(
        eei.load(load_address.into(), Wordsize::Halfword)?,
        15,
    );
    eei.set_x(dest, load_data);
    eei.increment_pc();
    Ok(())
}

pub fn execute_lw<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = eei.load(load_address.into(), Wordsize::Word)?;
    eei.set_x(dest, load_data);
    eei.increment_pc();
    Ok(())
}

pub fn execute_lbu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = eei.load(load_address.into(), Wordsize::Byte)?;
    eei.set_x(dest, load_data);
    eei.increment_pc();
    Ok(())
}

pub fn execute_lhu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: base,
        imm: offset,
        rd: dest,
    } = decode_itype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let load_address = base_address.wrapping_add(offset);
    let load_data = eei.load(load_address.into(), Wordsize::Halfword)?;
    eei.set_x(dest, load_data);
    eei.increment_pc();
    Ok(())
}

pub fn execute_sb_rv32i<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let SBtype {
        rs1: base,
        rs2: src,
        imm: offset,
    } = decode_stype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = eei.x(src);
    eei.store(store_address, store_data, Wordsize::Byte)?;
    eei.increment_pc();
    Ok(())
}

pub fn execute_sh_rv32i<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let SBtype {
        rs1: base,
        rs2: src,
        imm: offset,
    } = decode_stype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = eei.x(src);
    eei.store(store_address, store_data, Wordsize::Halfword)?;
    eei.increment_pc();
    Ok(())
}

pub fn execute_sw_rv32i<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let SBtype {
        rs1: base,
        rs2: src,
        imm: offset,
    } = decode_stype(instr);
    let base_address = eei.x(base);
    let offset = sign_extend(offset, 11);
    let store_address = base_address.wrapping_add(offset);
    let store_data = eei.x(src);
    eei.store(store_address, store_data, Wordsize::Word)?;
    eei.increment_pc();
    Ok(())
}

fn reg_imm_values<E: Eei>(eei: &E, instr: u32) -> (u32, u8, u32) {
    let Itype {
        rs1: src,
        imm: i_immediate,
        rd: dest,
    } = decode_itype(instr);
    let src: u32 = eei.x(src);
    let i_immediate = sign_extend(i_immediate, 11);
    (src, dest, i_immediate)
}

pub fn execute_addi<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = src.wrapping_add(i_immediate);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  
 
pub fn execute_slti<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = {
        let src: i32 = interpret_u32_as_signed!(src);
        let i_immediate: i32 = interpret_u32_as_signed!(i_immediate);
        (src < i_immediate) as u32
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_sltiu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = (src < i_immediate) as u32;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

pub fn execute_andi<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = src & i_immediate;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

pub fn execute_ori<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = src | i_immediate;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

pub fn execute_xori<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = src ^ i_immediate;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

pub fn execute_slli<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = src << (0x1f & i_immediate);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

pub fn execute_srli<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = src >> (0x1f & i_immediate);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

pub fn execute_srai<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src, dest, i_immediate) = reg_imm_values(eei, instr);
    let value = {
        let src: i32 = interpret_u32_as_signed!(src);
        interpret_i32_as_unsigned!(src >> (0x1f & i_immediate))
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}  

fn reg_reg_values<E: Eei>(eei: &E, instr: u32) -> (u32, u32, u8) {
    let Rtype {
        rs1: src1,
        rs2: src2,
        rd: dest,
    } = decode_rtype(instr);
    let src1: u32 = eei.x(src1);
    let src2: u32 = eei.x(src2);
    (src1, src2, dest)
}

pub fn execute_add<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1.wrapping_add(src2);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_sub<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1.wrapping_sub(src2);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_slt<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        (src1 < src2) as u32
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_sltu<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = (src1 < src2) as u32;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_and<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1 & src2;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_or<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1 | src2;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_xor<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1 ^ src2;
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_sll<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1 << (0x1f & src2);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_srl<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1 >> (0x1f & src2);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_sra<E :Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        interpret_i32_as_unsigned!(src1 >> (0x1f & src2))
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}








