use crate::{
    define_reg_reg_printer,
    hart::machine::Exception,
    instr_type::{decode_rtype, Rtype},
    interpret_u32_as_signed,
};

use super::{eei::Eei, Instr};

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

pub fn mul<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
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
    define_reg_reg_printer!("mul");
    Instr { executer, printer }
}

pub fn mulh<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
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
    define_reg_reg_printer!("mulh");
    Instr { executer, printer }
}

pub fn mulhsu<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
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
    define_reg_reg_printer!("mulhsu");
    Instr { executer, printer }
}

pub fn mulhu<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
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
    define_reg_reg_printer!("mulhu");
    Instr { executer, printer }
}

pub fn div<E: Eei>() -> Instr<E> {
    pub fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
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
    define_reg_reg_printer!("div");
    Instr { executer, printer }
}

pub fn divu<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
        let (src1, src2, dest) = reg_reg_values(eei, instr);
        let value = src1.wrapping_div(src2);
        eei.set_x(dest, value);
        eei.increment_pc();
        Ok(())
    }
    define_reg_reg_printer!("divu");
    Instr { executer, printer }
}

pub fn rem<E: Eei>() -> Instr<E> {
    pub fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
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
    define_reg_reg_printer!("rem");
    Instr { executer, printer }
}

pub fn remu<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
        let (src1, src2, dest) = reg_reg_values(eei, instr);
        let value = src1.wrapping_rem(src2);
        eei.set_x(dest, value);
        eei.increment_pc();
        Ok(())
    }
    define_reg_reg_printer!("remu");
    Instr { executer, printer }
}
