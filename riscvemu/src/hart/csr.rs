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

// Field masks for registers
pub const MSTATUS_MIE: u32 = 1 << 3;
pub const MSTATUS_MPIE: u32 = 1 << 7;
pub const MSTATUS_MPP: u32 = 0b11 << 11;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum CsrError {
    #[error("CSR 0x{0:x} does not exist (illegal instruction)")]
    NotPresentCsr(u16),
    #[error("Attempted write to read-only CSR (illegal instruction)")]
    ReadOnlyCsr,
    #[error("Attempted to write invalid value to WLRL field in CSR (illegal instruction)")]
    WlrlInvalidValue,
}

/*
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
 */

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
struct Csr {
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

impl Csr {
    /// Make a CSR where the whole u32 register is writable with any
    /// value
    fn new_all_values_allowed(initial_value: u32) -> Self {
        Self {
            write_mask: 0xffff_ffff,
            variable: initial_value,
            ..Self::default()
        }
    }

    /// Make a new CSR where the fields that are writable support all possible
    /// values, but which also contains some read-only fields with specified
    /// constant values. The value passed to initial_variable determines the
    /// initial value of the CSR (when combined with the constant part)
    fn new_masked(initial_variable: u32, constant: u32, write_mask: u32) -> Self {
        Self {
            write_mask,
            variable: initial_variable,
            constant: Some(constant),
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

/*
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
*/

/*
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
*/

#[derive(Debug, Copy, Clone)]
enum LowerUpper {
    Lower,
    Upper,
}

#[derive(Debug)]
enum ReadOnlyCsrRef {
    /// Stores a CSR that only holds a single read-only value.  Most
    /// often this value is zero.
    Constant(u32),
    /// Holds a reference to an entry in a vector of CSR
    /// objects. Although the underlying object may be writable, the
    /// register cannot be written through this reference.    
    Shadow(usize),
    /// Store a read-only view of mcycle (for cycle)
    Cycle(LowerUpper),
    /// Store a read-only view of minstret (for instret)
    Instret(LowerUpper),
    /// Indicates that time or timeh is requested. Use the inner enum
    /// to indicate whether the upper or lower 32 bits are requested
    Time(LowerUpper),
}

#[derive(Debug)]
enum ReadWriteCsrRef {
    /// Holds a reference to an entry in a vector of CSR objects. The
    /// CSR may be written through this reference.
    General(usize),
    /// Indicates that mcycle or mcycleh is requested. Use the inner enum
    /// to indicate whether the upper or lower 32 bits are requested
    MCycle(LowerUpper),
    /// Indicates that minstret or minstreth is requested. Use the inner enum
    /// to indicate whether the upper or lower 32 bits are requested
    MInstret(LowerUpper),
}

#[derive(Debug)]
enum CsrRef {
    ReadOnly(ReadOnlyCsrRef),
    ReadWrite(ReadWriteCsrRef),
}

/// Control and status registers (CSR)
///
/// Implements CSRs as documented in chapter 2 or the privileged
/// spec (v20211203)
///
#[derive(Debug, Default)]
pub struct CsrFile {
    /// The memory-mapped mtime register stores the 64-bit real
    /// time. The array is little-endian (mtime[0] is the lower
    /// 32 bits, and mtime[1] is the upper 32 bits). mtime is
    /// a read/write register that supports any value
    pub mtime: u64,

    /// For consistency, mtimecmp is stored here next to mtime (it
    /// is also a memory-mapped M-mode register). mtimecmp is a
    /// read/write register that supports any value
    pub mtimecmp: u64,

    /// Number of clock cycles executed by the hart
    pub mcycle: u64,

    /// Number of instructions retired by the hart
    pub minstret: u64,

    /// All CSRs are stored in a flat vector of objects which are all
    /// writable. The position of the CSR in this vector is _not_ its
    /// CSR address. This is stored in the map, which also controls
    /// whether the register is read-only or not. This allows multiple
    /// CSR addresses to refer to the same underlying CSR with
    /// different read/write properties.
    csr_data: Vec<Csr>,

    /// Map from CSR-address to underlying CSR, through a reference
    /// which controls read/write properties.
    addr_to_csr: HashMap<u16, CsrRef>,
}

fn add_read_write_csr(
    addr: u16,
    csr: Csr,
    csr_data: &mut Vec<Csr>,
    addr_to_csr: &mut HashMap<u16, CsrRef>,
) {
    let index = csr_data.len();
    csr_data.push(csr);
    addr_to_csr.insert(addr, CsrRef::ReadWrite(ReadWriteCsrRef::General(index)));
}

/// Panics if either the addr_to_shadow does not exist in the map. If
/// the shadowed register is a read-only constant, then a new
/// read-only entry is made with the same constant for the CSR at
/// addr, instead of making a shadow.
fn add_read_only_shadow(addr: u16, addr_to_shadow: u16, addr_to_csr: &mut HashMap<u16, CsrRef>) {
    match addr_to_csr
        .get(&addr_to_shadow)
        .expect("added csr previously, should be present")
    {
        CsrRef::ReadWrite(read_write_csr_ref) => match read_write_csr_ref {
            ReadWriteCsrRef::General(index) => {
                addr_to_csr.insert(addr, CsrRef::ReadOnly(ReadOnlyCsrRef::Shadow(*index)))
            }
            ReadWriteCsrRef::MCycle(lower_upper) => {
                addr_to_csr.insert(addr, CsrRef::ReadOnly(ReadOnlyCsrRef::Cycle(*lower_upper)))
            }
            ReadWriteCsrRef::MInstret(lower_upper) => addr_to_csr.insert(
                addr,
                CsrRef::ReadOnly(ReadOnlyCsrRef::Instret(*lower_upper)),
            ),
        },
        CsrRef::ReadOnly(read_only_csr_ref) => match read_only_csr_ref {
            ReadOnlyCsrRef::Constant(value) => {
                addr_to_csr.insert(addr, CsrRef::ReadOnly(ReadOnlyCsrRef::Constant(*value)))
            }
            _ => unimplemented!("shadows of other read-only registers are not supported"),
        },
    };
}

fn add_constant_csr(addr: u16, const_value: u32, addr_to_csr: &mut HashMap<u16, CsrRef>) {
    addr_to_csr.insert(
        addr,
        CsrRef::ReadOnly(ReadOnlyCsrRef::Constant(const_value)),
    );
}

/// Read the upper and lower 32-bit field of a u64 value
fn read_u64_field(value: &u64, lower_upper: LowerUpper) -> u32 {
    let field = match lower_upper {
        LowerUpper::Lower => extract_field(*value, 31, 0),
        LowerUpper::Upper => extract_field(*value, 63, 32),
    };
    field
        .try_into()
        .expect("32-bit field of u64 will fit in u32")
}

/// Write the upper and lower 32-bit field of a u64 value
fn write_u64_field(value: &mut u64, field: u32, lower_upper: LowerUpper) {
    match lower_upper {
        LowerUpper::Lower => {
            *value = (*value & 0xffff_ffff_0000_0000) | u64::from(field)
        }
        LowerUpper::Upper => {
            *value = (u64::from(field) << 32) | (*value & 0xffff_ffff)
        }
    }
}

impl CsrFile {
    /// Create CSRs for basic M-mode implementation
    ///
    /// Takes a references to the memory-mapped mtime registers, which
    /// is used as the basis for the unprivileged time CSR
    ///
    /// Pass in the read-only value mtvec, which stores the trap handler
    /// base address and mode.
    pub fn new_mmode(mtvec: u32) -> Self {
        let mut csr_data = Vec::new();
        let mut addr_to_csr = HashMap::new();

        // Machine information registers
        add_constant_csr(0xf11, 0, &mut addr_to_csr); // mvendorid
        add_constant_csr(0xf12, 0, &mut addr_to_csr); // marchid
        add_constant_csr(0xf13, 0, &mut addr_to_csr); // mimpid
        add_constant_csr(0xf14, 0, &mut addr_to_csr); // mhartid
        add_constant_csr(0xf15, 0, &mut addr_to_csr); // mconfigptr

        // Machine trap setup
        let write_mask = MSTATUS_MIE | MSTATUS_MPIE;
        let constant = MSTATUS_MPP; // can use mask as value should always be 0b11
        let mstatus = Csr::new_masked(0, constant, write_mask);
        add_read_write_csr(0x300, mstatus, &mut csr_data, &mut addr_to_csr);
        add_constant_csr(0x310, 0, &mut addr_to_csr); // mstatush

        let mie = Csr::new_all_values_allowed(0);
        add_read_write_csr(0x304, mie, &mut csr_data, &mut addr_to_csr);

        add_constant_csr(0x305, mtvec, &mut addr_to_csr); // mtvec
        add_constant_csr(0x310, 0, &mut addr_to_csr); // mstatush

        // Machine trap handling
        let mscratch = Csr::new_all_values_allowed(0);
        add_read_write_csr(0x340, mscratch, &mut csr_data, &mut addr_to_csr);

        let mepc = Csr::new_all_values_allowed(0); // todo: actually WARL, allowed addresses only
        add_read_write_csr(0x341, mepc, &mut csr_data, &mut addr_to_csr);

        let mcause = Csr::new_all_values_allowed(0); // todo: actually WLRL, allowed exceptions only
        add_read_write_csr(0x342, mcause, &mut csr_data, &mut addr_to_csr);

        add_constant_csr(0x343, 0, &mut addr_to_csr); // mtval

        let mip = Csr::new_all_values_allowed(0);
        add_read_write_csr(0x344, mip, &mut csr_data, &mut addr_to_csr);

        // Machine counters/timers
        addr_to_csr.insert(
            0xb00,
            CsrRef::ReadWrite(ReadWriteCsrRef::MCycle(LowerUpper::Lower)),
        ); // mcycle
        addr_to_csr.insert(
            0xb02,
            CsrRef::ReadWrite(ReadWriteCsrRef::MInstret(LowerUpper::Lower)),
        ); // minstret

        for n in 3..32 {
            add_constant_csr(0xb00 + n, 0, &mut addr_to_csr); // mhpmcountern
        }

        addr_to_csr.insert(
            0xb80,
            CsrRef::ReadWrite(ReadWriteCsrRef::MCycle(LowerUpper::Upper)),
        ); // mcycleh
        addr_to_csr.insert(
            0xb82,
            CsrRef::ReadWrite(ReadWriteCsrRef::MInstret(LowerUpper::Upper)),
        ); // minstreth

        for n in 3..32 {
            add_constant_csr(0xb80 + n, 0, &mut addr_to_csr); // mhpmcounternh
        }

        // Machine counter setup
        for n in 3..32 {
            add_constant_csr(0x320 + n, 0, &mut addr_to_csr); // mhpmeventn
        }

        // Unprivileged counters/timers
        add_read_only_shadow(0xc00, 0xb00, &mut addr_to_csr); // cycle
        addr_to_csr.insert(
            0xc01,
            CsrRef::ReadOnly(ReadOnlyCsrRef::Time(LowerUpper::Lower)),
        ); // time
        add_read_only_shadow(0xc02, 0xb02, &mut addr_to_csr); // instret

        for n in 3..32 {
            add_read_only_shadow(0xc00 + n, 0xb00 + n, &mut addr_to_csr);
            // hpmcountern
        }

        add_read_only_shadow(0xc80, 0xb80, &mut addr_to_csr); // cycleh
        addr_to_csr.insert(
            0xc81,
            CsrRef::ReadOnly(ReadOnlyCsrRef::Time(LowerUpper::Upper)),
        ); // timeh
        add_read_only_shadow(0xc82, 0xb82, &mut addr_to_csr); // instreth

        for n in 3..32 {
            add_read_only_shadow(0xc80 + n, 0xb80 + n, &mut addr_to_csr);
            // hpmcounternh
        }

        Self {
            csr_data,
            addr_to_csr,
            ..Self::default()
        }
    }

    fn csr_present(&self, addr: u16) -> bool {
        self.addr_to_csr.contains_key(&addr)
    }

    /// Read a value from a CSR
    ///
    /// If the CSR is not present, an error is returned. Otherwise,
    /// the contents of the CSR is returned. The caller can extract the
    /// bits or fields required.
    pub fn read(&self, addr: u16) -> Result<u32, CsrError> {
        if !self.csr_present(addr) {
            Err(CsrError::NotPresentCsr(addr))
        } else {
            let value = match self
                .addr_to_csr
                .get(&addr)
                .expect("should be present, we just checked")
            {
                CsrRef::ReadOnly(read_only_csr_ref) => match read_only_csr_ref {
                    ReadOnlyCsrRef::Constant(value) => *value,
                    ReadOnlyCsrRef::Cycle(lower_upper) => {
                        read_u64_field(&self.mcycle, *lower_upper)
                    }
                    ReadOnlyCsrRef::Time(lower_upper) => read_u64_field(&self.mtime, *lower_upper),
                    ReadOnlyCsrRef::Instret(lower_upper) => {
                        read_u64_field(&self.minstret, *lower_upper)
                    }
                    ReadOnlyCsrRef::Shadow(index) => {
                        let csr = &self.csr_data.get(*index).expect(
                            "index is always in bounds here, else addr_to_csr or csr_data is wrong",
                        );
                        csr.read()
                    }
                },
                CsrRef::ReadWrite(read_write_csr_ref) => match read_write_csr_ref {
                    ReadWriteCsrRef::General(index) => {
                        let csr = &self.csr_data.get(*index).expect(
                            "index is always in bounds here, else addr_to_csr or csr_data is wrong",
                        );
                        csr.read()
                    }
                    ReadWriteCsrRef::MCycle(lower_upper) => {
                        read_u64_field(&self.mcycle, *lower_upper)
                    }
                    ReadWriteCsrRef::MInstret(lower_upper) => {
                        read_u64_field(&self.minstret, *lower_upper)
                    }
                },
            };
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
    pub fn write(&mut self, addr: u16, value: u32) -> Result<(), CsrError> {
        if !self.csr_present(addr) {
            Err(CsrError::NotPresentCsr(addr))
        } else {
            match self
                .addr_to_csr
                .get(&addr)
                .expect("should be present, we just checked")
            {
                CsrRef::ReadOnly(_) => Err(CsrError::ReadOnlyCsr),
                CsrRef::ReadWrite(read_write_csr_ref) => match read_write_csr_ref {
                    ReadWriteCsrRef::General(index) => {
                        let csr = self.csr_data.get_mut(*index).expect(
                            "index is always in bounds here, else addr_to_csr or csr_data is wrong",
                        );
                        csr.write(value)
                    }
                    ReadWriteCsrRef::MCycle(lower_upper) => {
                        write_u64_field(&mut self.mcycle, value, *lower_upper);
                        Ok(())
                    }
                    ReadWriteCsrRef::MInstret(lower_upper) => {
                        write_u64_field(&mut self.minstret, value, *lower_upper);
                        Ok(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    /// Checks the machine information registers
    #[test]
    fn check_machine_information() {
        let csr_file = CsrFile::new_mmode(0);
        assert_eq!(csr_file.read(0xf11).expect("register present"), 0); // mvendorid
        assert_eq!(csr_file.read(0xf12).expect("register present"), 0); // marchid
        assert_eq!(csr_file.read(0xf13).expect("register present"), 0); // mimpid
        assert_eq!(csr_file.read(0xf14).expect("register present"), 0); // mhartid
        assert_eq!(csr_file.read(0xf15).expect("register present"), 0); // mconfigptr
    }

    /// Checks the register exists and returns zero
    #[test]
    fn check_other_read_only_zero() {
        let csr_file = CsrFile::new_mmode(0);
        assert_eq!(csr_file.read(0x310).expect("register present"), 0); // mstatush
        assert_eq!(csr_file.read(0x343).expect("register present"), 0); // mtval

        for n in 3..32 {
            assert_eq!(csr_file.read(0xb00 + n).expect("register present"), 0); // mhpmcountern
        }

        for n in 3..32 {
            assert_eq!(csr_file.read(0xb80 + n).expect("register present"), 0); // mhpmcounternh
        }

        for n in 3..32 {
            assert_eq!(csr_file.read(0x320 + n).expect("register present"), 0); // mhpmevent
        }

        for n in 3..32 {
            assert_eq!(csr_file.read(0xc00 + n).expect("register present"), 0); // hpmcountern
        }

        for n in 3..32 {
            assert_eq!(csr_file.read(0xc80 + n).expect("register present"), 0); // hpmcounternh
        }
    }

    /// Checks attempted writes to a selection of read-only registers
    /// returns an error
    #[test]
    fn check_error_on_write_to_read_only() {
        let mut csr_file = CsrFile::new_mmode(0);
        let result = csr_file.write(0xf11, 0); // mvendorid (read-only zero)
        assert_eq!(result, Err(CsrError::ReadOnlyCsr));

        let result = csr_file.write(0x310, 0); // mstatush
        assert_eq!(result, Err(CsrError::ReadOnlyCsr));

        let result = csr_file.write(0xf15, 0); // mconfigptr
        assert_eq!(result, Err(CsrError::ReadOnlyCsr));

        let result = csr_file.write(0xc00, 0); // cycle
        assert_eq!(result, Err(CsrError::ReadOnlyCsr));

        let result = csr_file.write(0xc01, 0); // time
        assert_eq!(result, Err(CsrError::ReadOnlyCsr));

        let result = csr_file.write(0xc02, 0); // instret
        assert_eq!(result, Err(CsrError::ReadOnlyCsr));
    }

    /// Checks attempted reads/writes to a non-existent registers
    /// returns an error
    #[test]
    fn check_error_on_access_to_not_present() {
        let mut csr_file = CsrFile::new_mmode(0);
        let result = csr_file.write(0x34b, 0); // mtval2 write
        assert_eq!(result, Err(CsrError::NotPresentCsr(0x34b)));
        let result = csr_file.read(0x34b); // mtval2 read
        assert_eq!(result, Err(CsrError::NotPresentCsr(0x34b)));

        let result = csr_file.write(0x3a0, 0); // pmpcfg0
        assert_eq!(result, Err(CsrError::NotPresentCsr(0x3a0)));
        let result = csr_file.read(0x3a0); // pmpcfg0
        assert_eq!(result, Err(CsrError::NotPresentCsr(0x3a0)));
    }

    #[test]
    fn check_mtime() {
        let mut csr_file = CsrFile::new_mmode(0);

        // Check time defaults to zero
        let time = csr_file.read(0xc01).unwrap();
        let timeh = csr_file.read(0xc81).unwrap();
        assert_eq!(time, 0);
        assert_eq!(timeh, 0);

        // Now set an arbitrary time and check
        csr_file.mtime = 0x1234_abcd_9876_cdef;
        let time = csr_file.read(0xc01).unwrap();
        let timeh = csr_file.read(0xc81).unwrap();
        assert_eq!(time, 0x9876_cdef);
        assert_eq!(timeh, 0x1234_abcd);
    }

    #[test]
    fn check_mcycle() {
        let mut csr_file = CsrFile::new_mmode(0);

        // Check cycle defaults to zero
        let mcycle = csr_file.read(0xb00).unwrap();
        let mcycleh = csr_file.read(0xb80).unwrap();
        assert_eq!(mcycle, 0);
        assert_eq!(mcycleh, 0);
        let cycle = csr_file.read(0xc00).unwrap();
        let cycleh = csr_file.read(0xc80).unwrap();
        assert_eq!(cycle, 0);
        assert_eq!(cycleh, 0);

        // Now set an arbitrary cycle and check
        csr_file.mcycle = 0x1234_abcd_9876_cdef;
        let cycle = csr_file.read(0xc00).unwrap();
        let cycleh = csr_file.read(0xc80).unwrap();
        assert_eq!(cycle, 0x9876_cdef);
        assert_eq!(cycleh, 0x1234_abcd);

	// Make a modification by writing to the mcycle CSR
	csr_file.write(0xb00, 0xeeee_ffff).unwrap();
	csr_file.write(0xb80, 0xaaaa_bbbb).unwrap();

	// Check the correct value
	assert_eq!(csr_file.mcycle, 0xaaaa_bbbb_eeee_ffff);
	
	// Check the value is correct after modifying
        let mcycle = csr_file.read(0xb00).unwrap();
        let mcycleh = csr_file.read(0xb80).unwrap();
	println!("{csr_file:x?},{mcycle:x}");
        assert_eq!(mcycle, 0xeeee_ffff);
        assert_eq!(mcycleh, 0xaaaa_bbbb);
        let cycle = csr_file.read(0xc00).unwrap();
        let cycleh = csr_file.read(0xc80).unwrap();
        assert_eq!(cycle, 0xeeee_ffff);
        assert_eq!(cycleh, 0xaaaa_bbbb);
    }

    #[test]
    fn check_minstret() {
        let mut csr_file = CsrFile::new_mmode(0);

        // Check instret defaults to zero
        let minstret = csr_file.read(0xb02).unwrap();
        let minstreth = csr_file.read(0xb82).unwrap();
        assert_eq!(minstret, 0);
        assert_eq!(minstreth, 0);
        let instret = csr_file.read(0xc02).unwrap();
        let instreth = csr_file.read(0xc82).unwrap();
        assert_eq!(instret, 0);
        assert_eq!(instreth, 0);

        // Now set an arbitrary instret and check
        csr_file.minstret = 0x1234_abcd_9876_cdef;
        let instret = csr_file.read(0xc02).unwrap();
        let instreth = csr_file.read(0xc82).unwrap();
	println!("{csr_file:x?},{instret}");
        assert_eq!(instret, 0x9876_cdef);
        assert_eq!(instreth, 0x1234_abcd);
	// Make a modification by writing to the minstret CSR
	csr_file.write(0xb02, 0xeeee_ffff).unwrap();
	csr_file.write(0xb82, 0xaaaa_bbbb).unwrap();

	// Check the correct value
	assert_eq!(csr_file.minstret, 0xaaaa_bbbb_eeee_ffff);
	
	// Check the value is correct after modifying
        let minstret = csr_file.read(0xb02).unwrap();
        let minstreth = csr_file.read(0xb82).unwrap();
	println!("{csr_file:x?},{minstret:x}");
        assert_eq!(minstret, 0xeeee_ffff);
        assert_eq!(minstreth, 0xaaaa_bbbb);
        let instret = csr_file.read(0xc02).unwrap();
        let instreth = csr_file.read(0xc82).unwrap();
        assert_eq!(instret, 0xeeee_ffff);
        assert_eq!(instreth, 0xaaaa_bbbb);
    }

}
