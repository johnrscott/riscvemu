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

    // If the destination is x0, do not perform the read at all (and
    // do not subsequently store the result. Cannot combine this read
    // with subsequent write in order to preserve ? before write.
    let csr_value = if dest != 0 {
	Some(eei.read_csr(csr)?)
    } else {
	None
    };

    let reg_value = eei.x(source);
    eei.write_csr(csr, reg_value)?;

    if let Some(csr_value) = csr_value {
	eei.set_x(dest, csr_value);
    }
    eei.increment_pc();
    Ok(())
}

pub fn execute_csrrs<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: source,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);
    let csr_value = eei.read_csr(csr)?;

    // Only perform the write if the source register is not x0
    if source != 0 {
	let reg_value = eei.x(source);
	
	// Modify CSR value by setting any bits
	// which are set in the source register
	let new_csr_value = csr_value | reg_value;
	
	eei.write_csr(csr, new_csr_value)?;
    }
    
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
    let csr_value = eei.read_csr(csr)?;

    // Only perform the write if the source register is not x0
    if source != 0 {
	let reg_value = eei.x(source);	
	
	// Modify CSR value by clearing any bits
	// which are set in the source register
	let new_csr_value = csr_value & !reg_value;
	
	eei.write_csr(csr, new_csr_value)?;
    }
    eei.set_x(dest, csr_value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_csrrwi<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: uimm,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);

    // If the destination is x0, do not perform the read at all (and
    // do not subsequently store the result. Cannot combine this read
    // with subsequent write in order to preserve ? before write.
    let csr_value = if dest != 0 {
	Some(eei.read_csr(csr)?)
    } else {
	None
    };

    eei.write_csr(csr, uimm.into())?;

    if let Some(csr_value) = csr_value {
	eei.set_x(dest, csr_value);
    }
    eei.increment_pc();
    Ok(())
}

pub fn execute_csrrsi<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: uimm,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);
    let csr_value = eei.read_csr(csr)?;

    // Only perform the write if the source register is not x0
    if uimm != 0 {
	// Modify CSR value by setting any bits
	// which are set in the source register
	let new_csr_value = csr_value | u32::from(uimm);
	
	eei.write_csr(csr, new_csr_value)?;
    }
    
    eei.set_x(dest, csr_value);
    eei.increment_pc();
    Ok(())
}

pub fn execute_csrrci<E: Eei>(eei: &mut E, instr: u32) -> Result<(), Exception> {
    let Itype {
        rs1: uimm,
        imm: csr,
        rd: dest,
    } = decode_itype(instr);
    let csr_value = eei.read_csr(csr)?;

    // Only perform the write if the source register is not x0
    if uimm != 0 {
	// Modify CSR value by clearing any bits
	// which are set in the source register
	let new_csr_value = csr_value & !u32::from(uimm);
	
	eei.write_csr(csr, new_csr_value)?;
    }
    eei.set_x(dest, csr_value);
    eei.increment_pc();
    Ok(())
}
