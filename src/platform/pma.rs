//! # Physical Memory Attributes
//!
//! This file defines the physical memory layout and attributes of the
//! RISC-V processor. It models a device that has a non-volatile memory
//! (like an EEPROM) for holding instructions, and a separate volatile
//! memory device for use as RAM while executing code.
//!
//! References to the privileged spec refer to version 20211203.
//!
//! ## Memory Map
//!
//! The memory map for the 32-bit physical address space of the
//! processor is as follows. Address ranges are listed in the format
//! A-B, where address A is the first byte of the region and address B
//! is the first byte above the region.
//!
//! When errors are returned by the PMA checker, they are checked in
//! this order:
//! * access (is the type of operation valid or not)
//! * alignment (if the access type is valid, is alignment condition met)
//!
//! For example, an attempt to fetch an instruction using a misaligned
//! address that pushes the prospective fetch slightly outside the
//! valid instruction region will cause an access fault, not an
//! alignment fault.
//!
//! ### Read/execute (non-volatile memory)
//!
//! This region of memory stores the trap vector table
//! (0x0000_0000-0x0000_0088), and approximately 4 MiB of instructions
//! (0x0000_0088-0x0040_0000) and read-only data. The interrupt vector
//! table reserves space for the full set of 32 interrupts.
//!
//! | Address | Width | Description |
//! |---------|-------|------------|
//! | 0x0000_0000 | 4 | Reset vector (pc points here on reset) |
//! | 0x0000_0004 | 4 | Non-maskable interrupt vector |
//! | 0x0000_0008 | 4 | Trap vector table base (exception vector) |
//! | 0x0000_0014 | 4 | Machine software interrupt vector |
//! | 0x0000_0024 | 4 | Machine timer interrupt vector |
//! | 0x0000_0034 | 4 | Machine external interrupt vector |
//!
//! Supported access types (section 3.6.2 privileged spec): execute
//! (i.e. instruction fetch) word (four bytes) is supported, and must
//! be four byte aligned. Load of byte, halfword, and word is supported,
//! with any alignment.
//!
//! These errors can be raised:
//! * Instruction address misaligned (on non-four-byte-aligned access)
//! * Store access fault (on attempt to store to this region)
//!
//! ### Vacant (between non-volatile memory and I/O memory)
//!
//! The region 0x0040_0000-0x1000_0000 is vacant (no read/write/execute).
//!
//! These errors can be raised:
//! * Instruction access fault (on attempt to fetch instruction from this region)
//! * Load access fault (on attempt to load from this region)
//! * Store access fault (on attempt to store to this region)
//!
//! ### Input/output and memory mapped registers (0x1000_0000-0x1000_0080)
//!
//! This region has space for 32 4-byte memory mapped input/output
//! devices (SPI and UART), memory mapped time registers, and
//! memory-mapped interrupt control. (Some registers are 8 bytes,
//! taking the space of two registers.)
//!
//! The mtime and mtimecmp registers (which control the timer
//! interrupt) are memory mapped. The softintctrl and extintctrl are
//! this platform's memory-mapped software and external interrupt
//! control registers (they are not part of the RISC-V specification,
//! but are what is referred to in section 3.1.9 of the privileged
//! spec regarding clearing mip.MSIP and mip.MEIP).
//!
//! | Address | Width | Description |
//! |---------|-------|-------------|
//! | 0x0001_0000 | 8 | mtime (64-bit real time) |
//! | 0x0001_0008 | 8 | mtimecmp (64-bit timer compare register) |
//! | 0x0001_0010 | 4 | softintctrl (32-bit software interrupt control register) |
//! | 0x0001_0014 | 4 | extintctrl (32-bit external interrupt control register) |
//! | 0x0001_0018 | 4 | uarttx (write causes low byte sent to UART; read as 0) |
//!
//! The region is read/write (but no instruction fetch); reads/writes
//! must be 4-byte width and be 4-byte aligned.
//!
//! These errors can be raised:
//! * Instruction access fault (on attempt to fetch instruction from this region)
//! * Load address misaligned
//! * Store address misaligned
//!
//! ### Vacant (between  I/O memory and RAM)
//!
//! The region 0x1000_0080-0x2000_0000 is vacant (no
//! read/write/execute). The same errors as listed above in the
//! previous vacant section are raised.
//!
//! ### 4 MiB Main memory (0x2000_0000-0x2040_0000)
//!
//! Main memory supports read/write of byte, halfword (2 bytes), and
//! word (4 bytes) access widths. Any alignment is valid.
//!
//! These errors can be raised:
//! * Instruction access fault (on attempt to fetch instruction from this region)
//! * Load address misaligned
//! * Store address misaligned
//!
//! ### Vacant (above 0x2040_0000)
//!
//! This is the region above RAM, and generates the same errors as the
//! vacant regions above.
//!

use super::machine::Exception;

pub const RESET_VECTOR: u32 = 0x0000_0000;
pub const NMI_VECTOR: u32 = 0x0000_0004;
pub const EXCEPTION_VECTOR: u32 = 0x0000_0008;
pub const MACHINE_SOFTWARE_INT_VECTOR: u32 = 0x0000_0014;
pub const MACHINE_TIMER_INT_VECTOR: u32 = 0x0000_0024;
pub const MACHINE_EXTERNAL_INT_VECTOR: u32 = 0x0000_0034;

pub const MTIME_ADDR: u32 = 0x1000_0000;
pub const MTIMEH_ADDR: u32 = 0x1000_0004;
pub const MTIMECMP_ADDR: u32 = 0x1000_0008;
pub const MTIMECMPH_ADDR: u32 = 0x1000_000c;
pub const SOFTINTCTRL_ADDR: u32 = 0x1000_0010;
pub const EXTINTCTRL_ADDR: u32 = 0x1000_0014;
pub const UARTTX_ADDR: u32 = 0x1000_0018;

/// Models the PMA checker (section 3.6 privileged spec)
///
/// Use this checker to test whether a memory access is going to be
/// allowed before attempting to perform it. In this emulator, the
/// memory itself is stored in a flat structure without memory
/// attributes. This checker is what imposes structure on the memory.
/// It is not possible to store the memory itself in this structure,
/// because of the side effects of writing to memory-mapped control
/// registers, which can affect other architectural state.
///
/// TODO conside moving the docs above to this struct.
#[derive(Debug)]
pub struct PmaChecker {
    eeprom_size: u32,
    ram_size: u32,
}

impl Default for PmaChecker {
    /// Defaults to 4 MiB EEPROM device size and 4 MiB RAM device size
    fn default() -> Self {
        Self::new(4 * 1024 * 1024, 4 * 1024 * 1024)
    }
}

impl PmaChecker {
    /// Pass the ROM device and RAM device size in bytes.
    pub fn new(eeprom_size: u32, ram_size: u32) -> Self {
        Self {
            eeprom_size,
            ram_size,
        }
    }

    /// You can only fetch instructions from the EEPROM region, and
    /// they must be four-byte aligned
    pub fn check_instruction_fetch(&self, addr: u32) -> Result<(), Exception> {
        if !self.in_eeprom(addr, 4) {
            // The only instruction-fetch region is the EEPROM region
            Err(Exception::InstructionAccessFault)
        } else if !address_aligned(addr, 4) {
            // Instruction fetches must be four-byte aligned
            Err(Exception::InstructionAddressMisaligned)
        } else {
            // Fetch will be valid
            Ok(())
        }
    }

    /// You can read from any region that is not vacant. I/O region
    /// reads must be four-byte aligned, but main memory reads and
    /// eeprom reads can have any alignment.
    pub fn check_load(&self, addr: u32, width: u32) -> Result<(), Exception> {
        if self.in_eeprom(addr, width) {
            // Any load from the eeprom region is allowed.
            Ok(())
        } else if self.in_io(addr, width) {
            // Load is from I/O region
            if width != 4 {
                // I/O load must have width 4
                Err(Exception::LoadAccessFault)
            } else if !address_aligned(addr, 4) {
                // I/O load must be four byte aligned
                Err(Exception::LoadAddressMisaligned)
            } else {
                Ok(())
            }
        } else if self.in_main_memory(addr, width) {
            // Load is from main memory
            if !main_memory_valid_width(width) {
                // Only byte, halfword or word loads are allowed
                Err(Exception::LoadAccessFault)
            } else {
                // Any alignment is allowed
                Ok(())
            }
        } else {
            // Loads are only allowed from I/O or main memory
            Err(Exception::LoadAccessFault)
        }
    }

    /// You can write to the I/O region or main memory. I/O region
    /// writes must be four-byte aligned, but main memory writes can have
    /// any alignment.
    pub fn check_store(&self, addr: u32, width: u32) -> Result<(), Exception> {
        if self.in_io(addr, width) {
            // Store is to I/O region
            if width != 4 {
                // I/O store must have width 4
                Err(Exception::StoreAccessFault)
            } else if !address_aligned(addr, 4) {
                // I/O store must be four byte aligned
                Err(Exception::StoreAddressMisaligned)
            } else {
                Ok(())
            }
        } else if self.in_main_memory(addr, width) {
            // Store is to main memory
            if !main_memory_valid_width(width) {
                // Only byte, halfword or word stores are allowed
                Err(Exception::StoreAccessFault)
            } else {
                // Any alignment is allowed
                Ok(())
            }
        } else {
            // Stores are only allowed to I/O or main memory
            Err(Exception::StoreAccessFault)
        }
    }

    /// True if address (and width) is fully in EEPROM region
    pub fn in_eeprom(&self, addr: u32, width: u32) -> bool {
        address_in_region(addr, width, 0x0000_0000, self.eeprom_size)
    }

    /// True if address (and width) is fully in I/O region
    fn in_io(&self, addr: u32, width: u32) -> bool {
        address_in_region(addr, width, 0x1000_0000, 0x1000_0080)
    }

    /// True if address (and width) is fully in main memory
    fn in_main_memory(&self, addr: u32, width: u32) -> bool {
        address_in_region(addr, width, 0x2000_0000, 0x2000_0000 + self.ram_size)
    }
}

/// Check width is byte, halfword or word
fn main_memory_valid_width(width: u32) -> bool {
    width == 1 || width == 2 || width == 4
}

/// Checks whether the area targeted by the address
/// and width fits in the region start-end
fn address_in_region(addr: u32, width: u32, start: u32, end: u32) -> bool {
    addr >= start && addr + width < end
}

/// Test if an address is aligned (add multiple of width)
fn address_aligned(addr: u32, width: u32) -> bool {
    addr % width == 0
}
