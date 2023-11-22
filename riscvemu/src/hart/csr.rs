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
//! CSRs). These are defined below. Registers not in the list below
//! are not present in the implementation (read/writes will trigger
//! illegal instruction).
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
//! mstatus: read/write, containing both WPRI and WARL fields. The
//  bit fields which are non-zero are as follows (assumes only M-mode):
//! - bit 3: MIE (interrupt enable)
//! - bit 7: MPIE (previous value of interrupt enable)
//! - bits [12:11]: MPP (previous privilege mode, always 0b11, WARL) (?)
//!
//! mstatush: upper 32-bit of status; all fields are read-only zero
//! (only little-endian memory is supported)
//!
//! mtvec: read-only, trap handler addresses
//! - bits [1:0]: 1 (vectored mode)
//! - bits [31:2]: trap vector table base address (4-byte aligned)
//!
//! mip: read/write register of pending interrupts. A pending
//! interrupt can be cleared by writing a 0 to that bit in the
//! register
//!
//! mie: read/write interrupt-enable register. To enable an interrupt
//! in M-mode, both mstatus.MIE and the bit in mie must be set. Bits
//! corresponding to interrupts that cannot occur must be read-only
//! zero.
//!
//! mcycle/mcycleh: read/write, 64-bit register (in two 32-bit
//! blocks), containing number of clock cycles executed by the
//! processor.
//!
//! minstret/minstreth: read/write, 64-bit register (in two 32-bit
//! blocks), containing number of instructions retired by the
//! processor.
//!
//! mhpmcounter[3-31]/mhpmcounter[3-31]h: both 32-bit read-only zero
//!
//! mhpmevent[3-31]: 32-bit, read-only zero
//!
//! Need to double check whether the following unprivileged CSRs are
//! required when only M-mode is implemented (i.e. in addition to the
//! m* versions).
//!
//! time/timeh: read-only shadows of the lower/upper 32-bit sections
//! of the mtime register.
//!
//! cycle/cycleh: read-only shadow of mcycle/mcycleh
//!
//! instret/instreth: read-only shadow of minstret/minstreth
//!
//! hpmcounter[3-31]/hpmcounter[3-31]h: read-only shadow of
//! mhpmcounter[3-31]/mhpmcounter[3-31]h
//!
//! mscratch: 32-bit read/write register
//!
//! mepc: 32-bit, read/write register (WARL, valid values are allowed
//! physical addresses).
//!
//! mcause: 32-bit, read/write, stores exception code and bit
//! indicating whether trap is interrupt. exception code is WLRL.
//!
//! mtval: read-only zero
//!
//! mconfigptr: read-only zero
//!
//! In addition to the CSRs listed above, the following two
//! memory-mapped registers exist:
//!
//! mtime: read/write 64-bit register incrementing at a constant rate
//!
//! mtimecmp: read/write 64-bit register that controls the timer
//! interrupt.
//!
//!

use std::collections::HashMap;
use thiserror::Error;

use crate::utils::extract_field;

#[derive(Debug, Error)]
pub enum CsrError {
    #[error("CSR 0x{0:x} does not exist (illegal instruction)")]
    NotPresentCsr(u16),
    #[error("Attempted write to read-only CSR (illegal instruction)")]
    ReadOnlyCsr,
    #[error("Attempted to write invalid value to WLRL field in CSR (illegal instruction)")]
    WlrlInvalidValue,
}

#[derive(Debug)]
enum ReadOnlyCsr {
    Constant(u32),
    /// For read-only shadows of other CSR registers
    CsrShadow(Box<Csr>),
    /// For read-only shadows of arbitrary references
    MemShadow(Box<u32>),
}

impl ReadOnlyCsr {
    fn read(&self) -> u32 {
        match self {
            Self::Constant(value) => *value,
            Self::CsrShadow(csr) => csr.read(),
            Self::MemShadow(value) => **value,
        }
    }
}

/// Check values to be written to writable CSR fields
///
/// This function takes a prospective value to be written to
/// a CSR and checks whether the values of writable fields
/// are valid. It only applies in cases where there exist fields
/// in the CSR that specify a limited set of legal values which
/// is not the full range of values that would fit in the field.
///
/// The function takes both the current value of the CSR and the
/// new value of the CSR. It returns a new value which has been
/// updated to reflect the current state of the CSR, if the write
/// would cause a WARL field to become invalid. It returns an error
/// if a write would cause a WLRL field to become invalid.
///
/// The latter takes precedence over the former, so a CSR with both
/// WARL and WLRL fields will not be written with a valid value to the
/// WARL field if the write to the WLRL field would cause an
/// error. This means that the illegal instruction that will
/// eventually result from the attempted CSR write will leave that CSR
/// unchanged.
///
/// This function is implemented on a per-CSR basis for the CSRs which
/// contains WARL and/or WRLR fields.
#[derive(Debug)]
struct WriteValueCheck(fn(current_value: u32, new_value: u32) -> Result<u32, CsrError>);

/// A writable CSR is one where at least some of the fields can be
/// written to. This is stored as a component storing the constant
/// (read-only, or only one legal value) part, and the variable
/// part.
///
#[derive(Debug, Default)]
struct WritableCsr {
    /// Stores (as ones) which portions of the CSR are writable
    /// (which means, in this context, which parts can be written with
    /// more than one legal value).
    write_mask: u32,
    /// Stores the variable (writable) part.
    variable: u32,
    /// Some writable fields in CSRs only support some values. If
    /// this CSR supports all values, this field is None. Otherwise,
    /// the field is a function that returns false if any of the
    /// fields written by
    write_value_check: Option<WriteValueCheck>,
    /// Stores the fixed part of a writable CSR. If the fixed part
    /// is always zero, then this is None
    constant: Option<u32>,
}

impl WritableCsr {
    /// Make a CSR where the whole u32 register is writable with any
    /// value
    fn new_all_values_allowed(initial_value: u32) -> Self {
        Self {
            write_mask: 0xffff_ffff,
            variable: initial_value,
            ..Self::default()
        }
    }

    /// Get the value of the CSR, by combining the constant and
    /// variable parts
    fn read(&self) -> u32 {
        self.constant.unwrap_or(0) | self.variable
    }

    /// Write a value to the CSR. The parts of the CSR that overlap
    /// with writable fields will be written, but the constant
    /// fields will be left the same.
    fn write(&mut self, mut value: u32) -> Result<(), CsrError> {
        if let Some(write_value_check) = &self.write_value_check {
            value = write_value_check.0(self.variable, value)?;
        }
        self.variable = value & self.write_mask;
        Ok(())
    }
}

#[derive(Debug)]
enum Csr {
    /// If the CSR is read-only, use this variant (which contains
    /// the fixed value of the register).
    ///
    /// Note that some CSR registers are read-only, even if they have
    /// addresses that imply they are read/write. For example, mtval
    /// is a read/write register, but may be implemented as read-only
    /// zero. TODO find where in the privileged spec it is explicitly
    /// stated that a read/write register implemented as read-only
    /// zero raises an illegal instruction on an attempted write, like
    /// other read-only registers.
    ///
    /// Item holds a reference to another CSR
    ReadOnly(ReadOnlyCsr),
    /// If the CSR contains writable fields, then use this variant
    Writable(WritableCsr),
}

impl Csr {
    /// Read the value of the CSR. Does not cause an error, because by
    /// this point, the CSR has been established as
    /// present. (Privilege is not considered in this M-mode
    /// implementation.)
    fn read(&self) -> u32 {
        match self {
            Self::ReadOnly(csr) => csr.read(),
            Self::Writable(csr) => csr.read(),
        }
    }

    /// Write a value to a CSR
    ///
    /// If the CSR is read-only, then an error will be returned
    ///
    /// What is actually written depends on the type of fields in the
    /// CSR. Read-only fields will retain their values. writable
    /// WLRL fields will be written provided that the value is legal;
    /// otherwise an error will be returned. If no error occurs:
    /// writable WARL fields will be updated if the value in the
    /// argument is legal, else they will retain their previous
    /// values; and writable fields that support all values will be
    /// updated unconditionally.
    fn write(&mut self, value: u32) -> Result<(), CsrError> {
        match self {
            Self::ReadOnly(_) => Err(CsrError::ReadOnlyCsr),
            Self::Writable(csr) => csr.write(value),
        }
    }
}

/// Control and status registers (CSR)
///
/// Implements CSRs as documented in chapter 2 or the privileged
/// spec (v20211203)
///
#[derive(Debug, Default)]
pub struct CsrFile {
    csrs: HashMap<u16, Box<Csr>>,
}

/// This is stored as two separate fields to allow passing a reference
/// to time and timeh to the relevant shadow CSRs
#[derive(Debug, Default)]
pub struct Time {
    lower: Box<u32>,
    upper: Box<u32>,
}

impl Time {
    pub fn time(&self) -> u64 {
        (u64::from(*self.upper) << 32) | u64::from(*self.lower)
    }

    pub fn set_time(&mut self, value: u64) {
        *self.lower = extract_field(value, 31, 0)
            .try_into()
            .expect("this will fit");
        *self.upper = extract_field(value, 63, 32)
            .try_into()
            .expect("this will fit");
    }

    pub fn increment_time(&mut self) {
        // This is inefficient
        self.set_time(self.time() + 1);
    }
}

impl CsrFile {
    /// Create CSRs for basic M-mode implementation
    ///
    /// Takes a references to the memory-mapped mtime registers, which
    /// is used as the basis for the unprivileged time CSR
    pub fn new_mmode(mtime: &Time) -> Self {
        let mut csrs = HashMap::new();

        // Machine counters
        let mcycle = Box::new(Csr::Writable(WritableCsr::new_all_values_allowed(0)));
        let mcycleh = Box::new(Csr::Writable(WritableCsr::new_all_values_allowed(0)));
        let minstret = Box::new(Csr::Writable(WritableCsr::new_all_values_allowed(0)));
        let minstreth = Box::new(Csr::Writable(WritableCsr::new_all_values_allowed(0)));
        let minstreth = Box::new(Csr::Writable(WritableCsr::new_all_values_allowed(0)));

        // // M-mode CSRs
        // csrs.insert(0xf11, 0); // mvendorid
        // csrs.insert(0xf12, 0); // marchid
        // csrs.insert(0xf13, 0); // mimpid
        // csrs.insert(0xf14, 0); // mhartid
        // csrs.insert(0xf15, 0); // mconfigptr
        // csrs.insert(0x300, 0); // mstatus
        // csrs.insert(0x304, 0); // mie
        // csrs.insert(0x305, 0); // mtvec
        // csrs.insert(0x310, 0); // mstatush
        // csrs.insert(0x340, 0); // mscratch
        // csrs.insert(0x341, 0); // mepc
        // csrs.insert(0x342, 0); // mcause
        // csrs.insert(0x343, 0); // mtval
        // csrs.insert(0x344, 0); // mip
        csrs.insert(0xb00, mcycle);
        // csrs.insert(0xb02, 0); // minstret
        // for n in 3..32 {
        //     csrs.insert(0xb00 + n, 0); // mhpmcountern
        // }
        // csrs.insert(0xb80, 0); // mcycleh
        // csrs.insert(0xb82, 0); // minstreth
        // for n in 3..32 {
        //     csrs.insert(0xb80 + n, 0); // mhpmcounternh
        // }
        // for n in 3..32 {
        //     csrs.insert(0x320 + n, 0); // mhpmeventn
        // }

        // Unprivileged read-only shadows
        let cycle = Box::new(Csr::ReadOnly(ReadOnlyCsr::CsrShadow(
            *csrs.get(&0xb00).expect("mcycle register is in map"),
        )));
        let cycleh = Csr::ReadOnly(ReadOnlyCsr::CsrShadow(mcycleh));
        let time = Csr::ReadOnly(ReadOnlyCsr::MemShadow(mtime.lower));
        let timeh = Csr::ReadOnly(ReadOnlyCsr::MemShadow(mtime.upper));
        let instret = Csr::ReadOnly(ReadOnlyCsr::CsrShadow(minstret));
        let instreth = Csr::ReadOnly(ReadOnlyCsr::CsrShadow(minstreth));

        // // Unprivileged CSRs
        csrs.insert(0xc00, cycle);
        // csrs.insert(0xc01, 0); // time
        // csrs.insert(0xc02, 0); // instret
        // for n in 3..32 {
        //     csrs.insert(0xc00 + n, 0); // hpmcountern
        // }
        // csrs.insert(0xc80, 0); // cycleh
        // csrs.insert(0xc81, 0); // timeh
        // csrs.insert(0xc82, 0); // instreth
        // for n in 3..32 {
        //     csrs.insert(0xc80 + n, 0); // hpmcounternh
        // }
	
        Self { csrs }
    }

    fn csr_present(&self, csr: u16) -> bool {
        self.csrs.contains_key(&csr)
    }

    /// Read a value from a CSR
    ///
    /// If the CSR is not present, an error is returned. Otherwise,
    /// the contents of the CSR is returned. The caller can extract the
    /// bits or fields required.
    pub fn read(&self, csr: u16) -> Result<u32, CsrError> {
        if !self.csr_present(csr) {
            Err(CsrError::NotPresentCsr(csr))
        } else {
            let value = self
                .csrs
                .get(&csr)
                .expect("should be present, we just checked")
                .read();
            Ok(value)
        }
    }

    /// Write a value from a CSR
    ///
    /// If the CSR is not present or is read-only, an error is returned.
    /// Then, value is written to the CSR, according to the following
    /// rules:
    /// - read-only fields in the CSR are preserved
    /// - writable fields in the CSR follow these behaviours:
    ///   - if all values are allowed, copy field from value
    ///   - if field is WLRL, return error on invalid value, state of
    ///     CSR remains unchanged. If value is valid, write it.
    ///   - if field is WARL, do not modify CSR on invalid value;
    ///     if value is valid, write it.
    ///
    pub fn write(&mut self, csr: u16, value: u32) -> Result<(), CsrError> {
        if !self.csr_present(csr) {
            Err(CsrError::NotPresentCsr(csr))
        } else {
            self.csrs
                .get_mut(&csr)
                .expect("should be present, we just checked")
                .write(value)
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
