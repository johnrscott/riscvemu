use crate::instr_type::{decode_utype, UJtype};

use super::eei::Eei;

/// Load upper immediate in 32-bit mode
///
/// Load the u_immediate into the upper 12 bits of the register
/// dest and fill the lower 20 bits with zeros. Set pc = pc + 4.
///
pub fn execute_lui_rv32i<E: Eei>(eei: &mut E, instr: u32) {
    let UJtype {
        rd: dest,
        imm: u_immediate,
    } = decode_utype(instr);
    eei.set_x(dest, u_immediate << 12);
    eei.increment_pc();
}
