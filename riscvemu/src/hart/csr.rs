//! Machine CSR Interface
//!
//!

use super::machine::Machine;

pub const CSR_MVENDORID: u32 = 0xf11;
pub const CSR_MARCHID: u32 = 0xf12;
pub const CSR_MIMPID: u32 = 0xf13;
pub const CSR_MHARTID: u32 = 0xf14;
pub const CSR_MCONFIGPTR: u32 = 0xf15;

pub const CSR_MSTATUS: u32 = 0x300;
pub const CSR_MISA: u32 = 0x301;
pub const CSR_MIE: u32 = 0x304;
pub const CSR_MTVEC: u32 = 0x305;
pub const CSR_MSTATUSH: u32 = 0x310;

pub const CSR_MSCRATCH: u32 = 0x340;
pub const CSR_MEPC: u32 = 0x341;
pub const CSR_MCAUSE: u32 = 0x342;
pub const CSR_MTVAL: u32 = 0x343;

pub const CSR_MCYCLE: u32 = 0xb00;
pub const CSR_MINSTRET: u32 = 0xb02;
pub const CSR_MHPMCOUNTER_BASE: u32 = 0xb00; // add 3..32 to get address
pub const CSR_MCYCLEH: u32 = 0xb80;
pub const CSR_MINSTRETH: u32 = 0xb82;
pub const CSR_MHPMCOUNTERH_BASE: u32 = 0xb80; // add 3..32 to get address

pub const CSR_MHPMEVENT_BASE: u32 = 0x320; // add 3..32 to get address

pub const CSR_CYCLE: u32 = 0xc00;
pub const CSR_TIME: u32 = 0xc01;
pub const CSR_INSTRET: u32 = 0xc02;
pub const CSR_HPMCOUNTER_BASE: u32 = 0xc00; // add 3..32 to get address
pub const CSR_CYCLEH: u32 = 0xc80;
pub const CSR_TIMEH: u32 = 0xc81;
pub const CSR_INSTRETH: u32 = 0xc82;

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
    ReadOnlyConstant(u32),
    
}

impl Csr {
    new_
}

pub struct MachineInterface {
    machine: Machine,
    addr_to_csr: HashMap<u16, Csr>

}
