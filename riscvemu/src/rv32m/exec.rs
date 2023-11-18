use crate::{hart::{ExecutionError, Hart}, instr_type::{decode_rtype, Rtype}, fields::sign_extend, interpret_u32_as_signed};

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


pub fn execute_mul_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
	let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1.wrapping_mul(src2) as u32
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_mulh_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
	let src1: i64 = interpret_u32_as_signed!(src1).into();
        let src2: i64 = interpret_u32_as_signed!(src2).into();
        (0xffff_ffff & (src1.wrapping_mul(src2) >> 32)).try_into().unwrap()
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())    
}

pub fn execute_mulhsu_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
	let src1: i64 = interpret_u32_as_signed!(src1).into();
        let src2: i64 = src2.into();
        (0xffff_ffff & (src1.wrapping_mul(src2) >> 32)).try_into().unwrap()
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())    
}

pub fn execute_mulhu_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
	let src1: u64 = src1.into();
        let src2: u64 = src2.into();
        (0xffff_ffff & (src1.wrapping_mul(src2) >> 32)).try_into().unwrap()
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_div_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
	let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
	// Put wrapping_div for consistency, but not sure what
	// wrapping div means for ints (same comment for rem)
        src1.wrapping_div(src2) as u32
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())    
}

pub fn execute_divu_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1.wrapping_div(src2);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())    
}

pub fn execute_rem_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = {
	let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1.wrapping_rem(src2) as u32
    };
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}

pub fn execute_remu_rv32m(hart: &mut Hart, instr: u32) -> Result<(), ExecutionError> {
    let (src1, src2, dest) = reg_reg_values(hart, instr)?;
    let value = src1.wrapping_rem(src2);
    hart.set_x(dest, value)?;
    hart.increment_pc();
    Ok(())
}
