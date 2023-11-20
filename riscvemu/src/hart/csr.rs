//! Control and status registers
//!
//! From the unprivileged spec version 20191213, chapter 9: "RISC-V
//! defines a separate address space of 4096 Control and Status
//! registers associated with each hart". These registers are mainly
//! associated with various privileged mode operations.
//!
//! Instructions for reading/writing CSRs are defined in chapter 9
//! (unprivileged spec v20191213), in the Zicsr extension. The
//! six instructions defined there allow various combinations of
//! reading/writing whole registers, or setting/clearing individual
//! sets of bits.
//!
//! The CSR registers themselves are defined in the privileged
//! specification. They are variable width, mapped at specific
//! specification-defined addresses, and contain fields with different
//! behaviour on read or write (this explanation ignores privilege,
//! assuming only M-mode is implemented):
//!
//! - not-present CSR: some CSRs are optional; if they are not present,
//!   an attempt to read or write results in illegal instruction.
//! - fully read-only CSR: a whole CSR may be defined as read-only. Attempts
//!   to write any portion of the register result in illegal instruction.
//! - partial read-only CSR: if some bits (but not all bits) of a CSR are
//!   read-only, then writes to the read-only bits are ignored.
//! - read/write fields that support any value: any value may be written
//!   to these fields without raising an exception.
//! - read/write fields that are reserved (WPRI): implementations should
//!   treat as read-only zero (a write is allowed, but has no effect).
//! - read/write fields where only some values are legal come in
//!   two variants:
//!   - WLRL means writes must be legal, otherwise non-legal may be
//!     returned on next read (implementation may also raise illegal
//!     instruction on the illegal write). The value returned on next
//!     read must deterministically depend on last write/previous value
//!     of CSR (in particular, it could be the illegal value just written?)
//!   - WARL means writes can be illegal values (no illegal instruction),
//!     but a legal value will always be read. The legal value read after
//!     an illegal write must depend deterministically on the illegal
//!     value just written and the state of the hart (in particular,
//!     it could be the previous legal value of the field?)
//!
//! Note that a field marked WPRI, a field marked WARL where the only
//! legal value is 0, and a read-only zero field all have the same
//! implementation (all allow writes and always return 0 when read).
//!
//! Most read/write instructions contain only one kind of field (WPRI,
//! WARL, or WLRL).
//!
//! If a write to one CSR changes the set of legal values of fields in
//! another CSR, the second CSR immediately adopts an unspecified
//! value from its new set of legal values.
//!
//! This file implements the CSRs of a simple RISC-V microcontroller
//! which only uses M-mode, and which uses only a minimal set of
//! CSRs). These are defined below:
//!
//! misa: read/write; single legal value 0 always returned (WARL),
//! meaning architecture is determined by non-standard means (it
//! is rv32im_zicsr implementing M-mode only).
//!
//! mvendorid: read-only; returns 0 to indicate not implemented.
//!
//! marchid: read-only; returns 0 to indicate not implemented.
//!
//! mimpid: read-only; returns 0 to indicate not implemented.
//! 
//! mhartid: read-only; returns 0 to indicate hart 0.
//!
//! mstatus: read/write, containing both WPRI and WARL fields.
//! 

use crate::utils::extract_field;

use super::memory::Memory;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsrError {
    #[error("CSR 0x{0:x} does not exist (illegal instruction)")]
    NonExistentCsr(u16),
    #[error("Attempted write to read-only CSR 0x{0:x} (illegal instruction)")]
    ReadOnlyCsr(u16),
    #[error("CSR 0x{0:x} required higher privilege (illegal instruction)")]
    PrivilegedCsr(u16),
}

/// Is the CSR read-only?
fn read_only_csr(csr: u16) -> bool {
    extract_field(csr, 11, 10) == 0b11 
}

/// Control and status registers (CSR)
///
/// Implements CSRs as documented in chapter 2 or the privileged
/// spec (v20211203)
///
#[derive(Debug, Default)]
pub struct Csr {
    memory: Memory,
}

impl Csr {
    /// Read a value from a CSR
    pub fn read(&mut self, csr: u16, value: u32) -> Result<u32, CsrError> {
	Ok(0)
    }

    /// Write a value from a CSR
    pub fn write(&mut self, csr: u16, value: u32) -> Result<u32, CsrError> {
	if read_only_csr(csr) {
	    Err(CsrError::ReadOnlyCsr(csr))
	} else {
	    Ok(0)
	}
    }   
}


// Machine-mode CSRs in 32-bit mode
//
// (Note: not all the CSRs are the same length in all modes. For example,
// many registers are MXLEN long (32-bit or 64-bit), but mvendorid is always
// 32-bit even in 64-bit mode.)
//
// misa: 32-bit, top two bits are 0b01 (for machine xlen 32-bit),
// bottom 26 bits contain extensions (0b1100 for RV32IM; bit 8 is I,
// bit 12 is M). So Register is 0x4000_1100. Value zero can also be
// returned to indicate not implemented.
//
// mvendorid: 32-bit, contains the vendor ID (JEDEC). Value zero returned to
// indicate register not implemented (or non-commercial implementation).
//
// marchid: 32-bit, specifies base microarchitecture. 
//
// mimpid: 32-bit, contains version of proessor implementation. Value zero
// means not implemented.
//
// mhartid: 32-bit, unique id of hart running code. If there is only one
// hart, this field is zero.
//
// mstatus(h): 32-bit machine status registers, mostly single bit
// fields indicating and controlling the state of the current harts
// operation.  mstatush is a 32-bit field containing a few extra
// fields that do not fit into the 32-bit mstatus register. This is a
// complex register due to the number of different behaviours relating
// to each bit; see section 3.1.6 in the privileged specification.
//
// mtvec: 32-bit register storing the address of trap handlers, and
// the mode (vectored or direct). May be implemented as read-only.
//
// medeleg/mideleg: do not exist if only M-mode is implemented (i.e.
// there is no S-mode)
//
// mip: 32-bit register storing pending interrupts. Bits 15:0 are
// standard interrupts defined in the specification, and 16 and above
// are for platform/custom use. A pending interrupt is cleared
// (i.e. after servicing the interrupt) by writing zero to the correct
// bit. An interrupt will cause a trap if the MIE in mstatus is set.
//
// mie: 32-bit interrupt enable register, not used when only M-mode is
// implemented (all interrupts are controlled by MIE in mstatus) (to
// double check).
//
// mcycle(h): 64-bit, number of clock cycles executed by the processor on
// which the hart is running. Power-on-reset value arbitrary, writable
// with an arbitrary value. Written value takes effect after writing
// instruction completes. In 32-bit mode, accessible as a low and high
// 32-bit register.
//
// minstret(h): 64-bit, number of instructions retired by the
// hart. Power-on-reset value arbitrary, writable with an arbitrary
// value. Written value takes effect after writing instruction
// completes. In 32-bit mode, accessible as a low and high
// 32-bit register. 
//
// mhpmcounter[3-31](h): 29 additional 64-bit event counter; required, but
// allowed to be read-only zero. In 32-bit mode, each 64-bit counter is
// accessible as a low and high 32-bit register.
//
// mhpevent[3-31]: 29 32-bit registers specifying what events are
// being counted by mhpmcounter* registers. Required, but allowed to
// be read-only zero.
//
// mcounteren: 32-bit register, with one bit for each of the 32 performance
// monitoring counters (including 
//
// mcounteren: 32-bit register to enable counters. Does not exist when only
// M-mode is implemented (i.e. there is no U-mode).
//
// mcounterinhibit: 32-bit register used to disable counters. Not required
// to be implemented.
// 
// mscratch: 32-bit read/write register for use by machine mode.
//
// mepc: 32-bit exception program counter. When a trap is taken to M-mode
// (i.e. when any trap is taken), this register stores the address of the
// instruction that encountered the trap.
//
// mcause: 32-bit register indicating the cause of the last event that
// caused a trap. Contains a bit which is set if the last event was an
// interrupt.
//
// mtval: 32-bit register to provide more information to a trap handler.
// May be implemented as read-only zero.
//
// mconfigptr: 32-bit register pointing to a data structure with
// information about the hart and the platform. May be implemented as
// read-only zero to indicate that the structure does not exist.
// 
