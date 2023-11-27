use crate::{hart::machine::Exception, instr_type::{Itype, decode_itype}};

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
    Ok(())
}
