//! Machine CSR Interface
//!
//!

use std::collections::HashMap;
use thiserror::Error;

use super::machine::Machine;

/// read-only; returns 0 to indicate not implemented.
pub const CSR_MVENDORID: u16 = 0xf11;

/// read-only; returns 0 to indicate not implemented.
pub const CSR_MARCHID: u16 = 0xf12;

/// read-only; returns 0 to indicate not implemented.
pub const CSR_MIMPID: u16 = 0xf13;

/// read-only; single hart system, returns 0 to indicate hart 0.
pub const CSR_MHARTID: u16 = 0xf14;

/// read-only zero, configuration platform-specification defined
pub const CSR_MCONFIGPTR: u16 = 0xf15;

/// mstatus: read/write, containing both WPRI and WARL fields. The
///  bit fields which are non-zero are as follows (assumes only M-mode):
/// - bit 3: MIE (interrupt enable)
/// - bit 7: MPIE (previous value of interrupt enable)
/// - bits [12:11]: MPP (previous privilege mode, always 0b11, WARL) (?)
pub const CSR_MSTATUS: u16 = 0x300;

/// read/write; single legal value 0 always returned (WARL), meaning
/// architecture is determined by non-standard means (it is
/// rv32im_zicsr implementing M-mode only).
pub const CSR_MISA: u16 = 0x301;

/// read/write interrupt-enable register. To enable an interrupt in
/// M-mode, both mstatus.MIE and the bit in mie must be set. Bits
/// corresponding to interrupts that cannot occur must be read-only
/// zero.
pub const CSR_MIE: u16 = 0x304;

/// read-only, trap handler vector table base address
/// - bits [1:0]: 1 (vectored mode)
/// - bits [31:2]: trap vector table base address (4-byte aligned)
pub const CSR_MTVEC: u16 = 0x305;

/// upper 32-bit of status; all fields are read-only zero (only
/// little-endian memory is supported)
pub const CSR_MSTATUSH: u16 = 0x310;

/// 32-bit read/write register for use by trap handlers
pub const CSR_MSCRATCH: u16 = 0x340;

/// 32-bit, read/write register (WARL, valid values are allowed
/// physical addresses). Stores the return-address from trap handler
pub const CSR_MEPC: u16 = 0x341;

/// 32-bit, read/write, stores exception code and bit indicating
/// whether trap is interrupt. exception code is WLRL.
pub const CSR_MCAUSE: u16 = 0x342;

/// read-only zero
pub const CSR_MTVAL: u16 = 0x343;

/// low 32 bits of read/write 64-bit register incrementing at a constant rate
pub const CSR_MCYCLE: u16 = 0xb00;

/// low 32 bits of read/write, 64-bit register containing number of
/// instructions retired by the processor.
pub const CSR_MINSTRET: u16 = 0xb02;

/// read-only zero
pub const CSR_MHPMCOUNTER_BASE: u16 = 0xb00; // add 3..32 to get address

/// low 32 bits of read/write, 64-bit register containing number of
/// clock cycles executed by the processor.
pub const CSR_MCYCLEH: u16 = 0xb80;

/// low 64 bits of read/write, 64-bit register containing number of
/// instructions retired by the processor.
pub const CSR_MINSTRETH: u16 = 0xb82;

/// read-only zero
pub const CSR_MHPMCOUNTERH_BASE: u16 = 0xb80; // add 3..32 to get address

/// read-only zero
pub const CSR_MHPMEVENT_BASE: u16 = 0x320; // add 3..32 to get address

/// read-only shadow of lower 32 bits of 64-bit mcycle
pub const CSR_CYCLE: u16 = 0xc00;

/// read-only shadow of lower 32 bits of memory mapped 64-bit mtime
pub const CSR_TIME: u16 = 0xc01;

/// read-only shadow of minstret
pub const CSR_INSTRET: u16 = 0xc02;

/// read-only zero
pub const CSR_HPMCOUNTER_BASE: u16 = 0xc00; // add 3..32 to get address

/// read-only shadow of upper 32 bits of 64-bit mcycle
pub const CSR_CYCLEH: u16 = 0xc80;

/// read-only shadow of upper 32 bits of memory mapped 64-bit mtime
pub const CSR_TIMEH: u16 = 0xc81;

/// read-only shadow of upper 32 bits of 64-bit minstret
pub const CSR_INSTRETH: u16 = 0xc82;

/// read-only zero
pub const CSR_HPMCOUNTERH_BASE: u16 = 0xc80; // add 3..32 to get address

#[derive(Debug, Error)]
enum CsrError {
    #[error("CSR 0x{0:x} is not present")]
    NotPresent(u16),
}

/// Read a CSR (already established to exist)
struct ReadCsr(fn(&Machine) -> u32);

/// Write a CSR (can return an error if a WRLR write would be invalid)
struct WriteCsr(fn(&mut Machine, value: u32) -> Result<(), CsrError>);

/// Control and status registers
///
/// In this implementation, there are the following kinds of
/// control and status registers:
/// - read-only constant CSRs (most often zero). Attempting
///   to write to these raises an error.
/// - read/write CSRs whose fields are writable WARL, writable
///   with any value, or read-only. These CSRs do not raise
///   errors on writes, but writes to read-only fields are
///   ignored, and invalid writes to WARL fields cause that
///   field to remain unchanged compared to before the write.
/// - read/write fields with at least one writable WRLR field.
///   If this field is written with an invalid value, then
///   an error is returned and the CSR is not modified (even
///   if other fields would be written with legal values).
///
enum Csr {
    Constant(u32),
    ReadOnly(ReadCsr),
    ReadWrite(ReadCsr, WriteCsr),
}

impl Csr {
    fn new_constant(value: u32) -> Self {
	Self::Constant(value)
    }

    fn new_read_only(read_csr: ReadCsr) -> Self {
	Self::ReadOnly(read_csr)
    }

    fn new_read_write(read_csr: ReadCsr, write_csr: WriteCsr) -> Self {
	Self::ReadWrite(read_csr, write_csr)
    }

}

#[derive(Default)]
pub struct MachineInterface {
    machine: Machine,
    addr_to_csr: HashMap<u16, Csr>,
}

impl MachineInterface {
    pub fn new() -> Self {
        let mut addr_to_csr = HashMap::new();

        addr_to_csr.insert(CSR_MVENDORID, Csr::new_constant(0));
        addr_to_csr.insert(CSR_MARCHID, Csr::new_constant(0));
        addr_to_csr.insert(CSR_MIMPID, Csr::new_constant(0));
        addr_to_csr.insert(CSR_MHARTID, Csr::new_constant(0));
        addr_to_csr.insert(CSR_MCONFIGPTR, Csr::new_constant(0));
        addr_to_csr.insert(
            CSR_MSTATUS,
            Csr::new_read_write(ReadCsr(read_mstatus), WriteCsr(write_mstatus)),
        );
        addr_to_csr.insert(CSR_MISA, Csr::new_constant(0));
        // mie
        addr_to_csr.insert(
            CSR_MISA,
            Csr::new_read_only(ReadCsr(|machine: &Machine| machine.trap_ctrl.csr_mtvec())),
        );
        addr_to_csr.insert(CSR_MSTATUSH, Csr::new_constant(0));
        addr_to_csr.insert(
            CSR_MSCRATCH,
            Csr::ReadWrite(
                ReadCsr(|machine: &Machine| machine.mscratch),
                WriteCsr(|machine: &mut Machine, value: u32| {
                    machine.mscratch = value;
                    Ok(())
                }),
            ),
        );
        // mepc
        // mcause
        addr_to_csr.insert(CSR_MTVAL, Csr::new_constant(0));
        // mcycle
        // minstret
        // mcycleh
        // minstreth
        // cycle
        // time
        // instret
        // cycleh
        // timeh
        // instreth

        for n in 3..32 {
            addr_to_csr.insert(CSR_MHPMCOUNTER_BASE + n, Csr::new_constant(0));
            addr_to_csr.insert(CSR_MHPMCOUNTERH_BASE + n, Csr::new_constant(0));
            addr_to_csr.insert(CSR_MHPMEVENT_BASE + n, Csr::new_constant(0));
            addr_to_csr.insert(CSR_HPMCOUNTER_BASE + n, Csr::new_constant(0));
            addr_to_csr.insert(CSR_HPMCOUNTERH_BASE + n, Csr::new_constant(0));
        }

        Self {
            addr_to_csr,
            ..Self::default()
        }
    }
}

fn read_mstatus(machine: &Machine) -> u32 {
    0
}

fn write_mstatus(machine: &mut Machine, value: u32) -> Result<(), CsrError> {
    Ok(())
}
