use crate::{
    hart::machine::Exception,
    instr_type::{decode_itype, Itype},
};

use super::eei::Eei;

pub fn execute_csrrw<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: source,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);
    let reg_value = eei.x(source);
    let csr_value = eei.read_csr(csr)?;
    eei.write_csr(csr, reg_value)?;
    eei.set_x(dest, csr_value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_csrrs<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: source,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);
    let reg_value = eei.x(source);
    let csr_value = eei.read_csr(csr)?;

    // Modify CSR value by setting any bits
    // which are set in the source register
    let new_csr_value = csr_value | reg_value;

    eei.write_csr(csr, new_csr_value)?;
    eei.set_x(dest, csr_value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_csrrc<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: source,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);
    let reg_value = eei.x(source);
    let csr_value = eei.read_csr(csr)?;

    // Modify CSR value by clearing any bits
    // which are set in the source register
    let new_csr_value = csr_value & !reg_value;

    eei.write_csr(csr, new_csr_value)?;
    eei.set_x(dest, csr_value);
    eei.increment_pc();
    Ok(())
}
