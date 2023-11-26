use crate::{
    hart::machine::Exception,
    instr_type::{decode_rtype, Rtype},
    interpret_u32_as_signed,
};

use super::eei::Eei;

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

pub fn execute_mul<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1.wrapping_mul(src2) as u32
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_mulh<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i64 = interpret_u32_as_signed!(src1).into();
        let src2: i64 = interpret_u32_as_signed!(src2).into();
        (0xffff_ffff & (src1.wrapping_mul(src2) >> 32))
            .try_into()
            .unwrap()
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_mulhsu<E: Eei>(
    eei: &mut E,
    instr: u32,
) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i64 = interpret_u32_as_signed!(src1).into();
        let src2: i64 = src2.into();
        (0xffff_ffff & (src1.wrapping_mul(src2) >> 32))
            .try_into()
            .unwrap()
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_mulhu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: u64 = src1.into();
        let src2: u64 = src2.into();
        (0xffff_ffff & (src1.wrapping_mul(src2) >> 32))
            .try_into()
            .unwrap()
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_div<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        // Put wrapping_div for consistency, but not sure what
        // wrapping div means for ints (same comment for rem)
        src1.wrapping_div(src2) as u32
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_divu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1.wrapping_div(src2);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_rem<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = {
        let src1: i32 = interpret_u32_as_signed!(src1);
        let src2: i32 = interpret_u32_as_signed!(src2);
        src1.wrapping_rem(src2) as u32
    };
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_remu<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let (src1, src2, dest) = reg_reg_values(eei, instr);
    let value = src1.wrapping_rem(src2);
    eei.set_x(dest, value);
    eei.increment_pc();
    Ok(())
}
