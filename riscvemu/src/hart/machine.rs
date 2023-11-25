//! M-mode Implementation
//!
//! This file contains the implementation of a basic M-mode-only
//! RISC-V machine.
//!
//! References to the privileged spec refer to version 20211203.
//!

use thiserror::Error;

use crate::utils::mask;

/// Mask for valid bits in mcause. The mcause register is WARL,
/// so writes will be masked before writing to mcause.
pub const MCAUSE_MASK: u32 = 0x8000_08ff;

#[derive(Copy, Clone)]
pub enum Exception {
    InstructionAddressMisaligned,
    InstructionAccessFault,
    IllegalInstruction,
    Breakpoint,
    LoadAddressMisaligned,
    LoadAccessFault,
    StoreAddressMisaligned,
    StoreAccessFault,
    MmodeEcall,
}

/// All machine-level interrupts
#[derive(Copy, Clone)]
pub enum Interrupt {
    Software,
    Timer,
    External,
}

#[derive(Copy, Clone)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

// mip fields
pub const MIP_MSIP: u32 = 3;
pub const MIP_MTIP: u32 = 7;
pub const MIP_MEIP: u32 = 11;

// mstatus fields
pub const MSTATUS_MIE: u32 = 3;
pub const MSTATUS_MPIE: u32 = 7;
pub const MSTATUS_MPP: u32 = 11;

impl Trap {
    /// The value of the mcause CSR for this trap
    pub fn mcause(&self) -> u32 {
        self.interrupt_bit() | self.cause()
    }

    /// Returns the interrupt-bit component of mcause
    pub fn interrupt_bit(&self) -> u32 {
        match self {
            Self::Interrupt(_) => 0x8000_0000,
            Self::Exception(_) => 0x0000_0000,
        }
    }

    /// The exception code for an interrupt is the bit position
    /// in mie and mip used to enable the interrupt and report it
    /// as pending.
    pub fn cause(&self) -> u32 {
        match self {
            Self::Interrupt(int) => match int {
                Interrupt::Software => MIP_MSIP,
                Interrupt::Timer => MIP_MTIP,
                Interrupt::External => MIP_MEIP,
            },
            Self::Exception(ex) => match ex {
                Exception::InstructionAddressMisaligned => 0,
                Exception::InstructionAccessFault => 1,
                Exception::IllegalInstruction => 2,
                Exception::Breakpoint => 3,
                Exception::LoadAddressMisaligned => 4,
                Exception::LoadAccessFault => 5,
                Exception::StoreAddressMisaligned => 6,
                Exception::StoreAccessFault => 7,
                Exception::MmodeEcall => 11,
            },
        }
    }
}

#[derive(Debug, Error)]
pub enum TrapCtrlError {
    #[error("Trap vector base address must be four byte aligned")]
    TrapVectorBaseMisaligned,
    #[error("Physical memory address width must be no more than 32 bits")]
    PhysicalMemoryTooLarge,
}

#[derive(Debug, Default)]
struct TimerInterrupt {
    /// Timer interrupt enable
    mtie: bool,
    /// Real time
    mtime: u64,
    /// Timer compare register, used to control timer
    /// interrupt
    mtimecmp: u64,
}

impl TimerInterrupt {
    /// Return the MTIP bit. This function also evaluates
    /// the bit, which is equal to mtime >= mtimecmp (see
    /// section 3.1.2 privileged spec). Although 3.1.9
    /// appears to state that writing mtimecmp clears
    /// MTIP, this is interpreted as meaning mtimecmp
    /// _can_ clear MTIP.
    fn mtip(&self) -> bool {
        self.mtime >= self.mtimecmp
    }
}

/// Trap control
///
/// This implementation uses
#[derive(Debug, Default)]
pub struct TrapCtrl {
    /// Global interrupt enable bit in mstatus (MIE)
    mstatus_mie: bool,
    /// Previous global interrupt enable bit in mstatus (MPIE)
    mstatus_mpie: bool,
    /// The trap cause register
    mcause: u32,
    /// Trap vector table base address. Must be four-byte aligned.
    /// Trap vectors are 32-bit addresses to functions that will
    /// handle the trap. The single handler for all exception
    /// traps is located at trap_vector_base. Interrupts are handled
    /// by functions at offsets trap_vector_base + 4*cause.
    trap_vector_base: u32,
    /// When control is transferred to a trap, this register stores the
    /// address of the instruction that was interrupt or that encountered
    /// the exception (privileged spec, p. 38)
    mepc: u32,
    /// Valid mepc values are 4-byte aligned, and do not exceed valid
    /// address range for physical addresses (see the
    /// physical_address_bits argument).
    mepc_mask: u32,
    /// Timer registers and interrupt
    timer_interrupt: TimerInterrupt,
    /// External interrupt pending bit
    meip: bool,
    /// External interrupt enable bit
    meie: bool,
    /// Machine software interrupt pending
    msip: bool,
    /// Machine software interrupt enable
    msie: bool,
}

pub const MTVEC_MODE_VECTORED: u32 = 1;

impl TrapCtrl {
    pub fn set_mtimecmp(&mut self, value: u64) {
        self.timer_interrupt.mtimecmp = value
    }

    pub fn mtimecmp(&mut self) -> u64 {
        self.timer_interrupt.mtimecmp
    }

    pub fn increment_mtime(&mut self) {
        self.timer_interrupt.mtime += 1;
    }

    pub fn raise_external_interrupt(&mut self) {
        self.meip = true
    }

    pub fn clear_external_interrupt(&mut self) {
        self.meip = false
    }

    pub fn raise_software_interrupt(&mut self) {
        self.msip = true
    }

    pub fn clear_software_interrupt(&mut self) {
        self.msip = false
    }

    /// Write the mstatus register
    ///
    /// The only writable bits are bit 3 (MIE) and bit 7 (MPIE).
    /// Other bits are read-only zero, apart from MTIP which remains
    /// 0b11. Writing an invalid value will not cause an error (WARL),
    /// but will only update bit 3 and bit 7
    pub fn csr_write_mstatus(&mut self, value: u32) {
        self.mstatus_mie = value >> MSTATUS_MIE & 1 != 0;
        self.mstatus_mpie = value >> MSTATUS_MPIE & 1 != 0;
    }

    /// Construct the mstatus register for reading
    pub fn csr_mstatus(&self) -> u32 {
        (self.mstatus_mie as u32) << MSTATUS_MIE
            | (self.mstatus_mpie as u32) << MSTATUS_MPIE
            | 0b11 << MSTATUS_MPP
    }

    /// In this implementation, the mtvec CSR is read-only
    pub fn csr_mtvec(&self) -> u32 {
        (self.trap_vector_base << 2) | MTVEC_MODE_VECTORED
    }

    /// Get the mip (interrupt pending) status register
    pub fn csr_mip(&self) -> u32 {
        (self.msip as u32) << MIP_MSIP
            | (self.timer_interrupt.mtip() as u32) << MIP_MTIP
            | (self.meip as u32) << MIP_MEIP
    }

    /// Get the mie (interrupt enable) register
    pub fn csr_mie(&self) -> u32 {
        // Note bit positions are the same as in mip
        (self.msie as u32) << MIP_MSIP
            | (self.timer_interrupt.mtie as u32) << MIP_MTIP
            | (self.meie as u32) << MIP_MEIP
    }

    /// Write the mie register
    pub fn csr_write_mie(&mut self, value: u32) {
        self.msie = value >> MIP_MSIP & 1 != 0;
        self.timer_interrupt.mtie = value >> MIP_MTIP & 1 != 0;
        self.meie = value >> MIP_MEIP & 1 != 0;
    }

    /// Get the mepc (return address after trap) register
    pub fn csr_mepc(&self) -> u32 {
        self.mepc
    }

    /// Set the mepc (return address after trap) register
    ///
    /// Only valid addresses for mepc are legal values.  valid
    /// addresses are four-byte aligned, and are valid physical memory
    /// addresses. The value is masked so that it becomes valid,
    /// and then written to mepc. The field is WARL, so a write
    /// of an invalid value does not raise an error.
    pub fn csr_write_mepc(&mut self, value: u32) {
        self.mepc = self.mepc_mask & value;
    }

    /// Get the mcause (cause of trap) register
    pub fn csr_mcause(&self) -> u32 {
        self.mcause
    }

    pub fn csr_write_mcycle(&mut self) -> u32 {
        low_word(&self.timer_interrupt.mtime)
    }

    pub fn mmap_mtime(&self) -> u32 {
        low_word(&self.timer_interrupt.mtime)
    }

    pub fn mmap_mtimeh(&self) -> u32 {
        high_word(&self.timer_interrupt.mtime)
    }

    /// Set the mcause (cause of trap) register
    ///
    /// The register has all-WARL fields, so an invalid
    /// value will be converted to a valid value by a mask
    /// before the write. The resulting value of the register
    /// will agree with the written value for the valid bits,
    /// and invalid bits will continue to be zero.
    ///
    /// Valid bits are as follows:
    /// - bit 0: instruction address misaligned
    /// - bit 1: instruction access fault
    /// - bit 2: illegal instruction
    /// - bit 3: machine software interrupt/breakpoint
    /// - bit 4: load address misaligned
    /// - bit 5: load access fault
    /// - bit 6: store address misaligned
    /// - bit 7: store access fault/machine timer interrupt
    /// - bit 11: ecall from M-mode/machine external interrupt
    /// - bit 31: interrupt flag
    ///
    /// The resulting mask is 0b8000_08ff.
    ///
    pub fn csr_write_mcause(&mut self, value: u32) {
        self.mcause = MCAUSE_MASK & value;
    }

    /// For an exception, return the trap vector base address. For an
    /// interrupt, return a trap vector based on the cause.
    pub fn trap_vector_address(&self, trap: Trap) -> u32 {
        match &trap {
            Trap::Exception(_) => self.trap_vector_base,
            Trap::Interrupt(int) => self.trap_vector_base + 4 * trap.cause(),
        }
    }

    /// Make a new trap control struct
    ///
    /// Returns an error if the trap_vector_base address is not
    /// four-byte aligned
    fn new(trap_vector_base: u32, physical_address_bits: u32) -> Result<Self, TrapCtrlError> {
        if trap_vector_base % 4 != 0 {
            Err(TrapCtrlError::TrapVectorBaseMisaligned)
        } else if physical_address_bits >= 32 {
            Err(TrapCtrlError::PhysicalMemoryTooLarge)
        } else {
            Ok(Self {
                trap_vector_base,
                mepc_mask: mask(physical_address_bits) & 0xffff_fffc,
                ..Self::default()
            })
        }
    }

    /// As per section 3.1.6.1 privileged spec, MIE bits is saved
    /// to MPIE on a trap, and MIE is set to 0
    fn save_mie_bit(&mut self) {
        self.mstatus_mpie = self.mstatus_mie;
        self.mstatus_mie = false;
    }

    /// As per section 3.1.6.1 privileged spec, MPIE bits is restored
    /// to MIE on an mret, and MPIE is set to 1
    fn restore_mie_bit(&mut self) {
        self.mstatus_mie = self.mstatus_mpie;
        self.mstatus_mpie = true;
    }

    /// Evaluate the conditions for trapping an interrupt
    ///
    /// The conditions for whether the interrupt for cause i is raised
    /// are laid out on p. 32 of the privileged spec: for an
    /// M-only-mode processor, if the mstatus.MIE bit is set
    /// (interrupts are globally enabled); and bit i is set in mip and
    /// mie, then the interrupt is trapped.
    ///
    /// In order for these conditions to be evaluated within a bounded
    /// amount of time from when the interrupt becomes pending
    /// (p. 32), this function should be called at the beginning of
    /// each instruction cycle.
    ///
    /// The conditions for raising an interrupt trap are evaluated
    /// in the order: MEI (external); MSI (software); MTI (timer). The
    /// first interrupt satisfying the conditions above is trapped.
    ///
    /// If an interrupt trap should be taken, the current program
    /// counter (passed as an argument) is stored in mepc (this is the
    /// address of the instruction that was interrupted; p. 38 of the
    /// privileged spec), and the function will return the address of
    /// the entry in the trap vector table where the address of the
    /// handler is stored; else it will return None.
    ///
    /// If an address is returned, set the program counter to the
    /// result of reading that memory address.
    ///
    pub fn trap_interrupt(&mut self, pc: u32) -> Option<u32> {
        // Do not modify order
        self.interrupt_should_trap(Interrupt::External, pc)?;
        self.interrupt_should_trap(Interrupt::Software, pc)?;
        self.interrupt_should_trap(Interrupt::Timer, pc)?;
        None
    }

    /// Raise an exception
    ///
    /// Unlike an interrupt, an exception occurs as a result of an
    /// attempted action by an instruction, while the instruction is
    /// mid-execution. If an exceptional condition is detected, call
    /// this function to store the current program counter (of the
    /// instruction causing the exception) to mepc, set the mcause
    /// register to the cause of the exception, and obtain the address
    /// of an entry in the trap vector table.
    ///
    /// Using the memory address returned by this function, set the
    /// program counter to the result of reading that memory address.
    pub fn raise_exception(&mut self, pc: u32, ex: Exception) -> u32 {
        let trap = Trap::Exception(ex);
        self.mcause = trap.mcause();
        self.mepc = pc;
        self.trap_vector_address(trap)
    }

    /// Return from an exception or interrupt
    ///
    /// Restore the mstatus.MIE bit and return the value stored in
    /// mepc (see p. 47 privileged spec).
    ///
    /// Write the value returned by this function to the program
    /// counter and resume execution.
    pub fn mret(&mut self) -> u32 {
        self.restore_mie_bit();
        self.mepc
    }

    fn interrupts_globally_enabled(&self) -> bool {
        self.mstatus_mie
    }

    fn interrupt_enabled(&self, int: Interrupt) -> bool {
        self.interrupts_globally_enabled()
            && match int {
                Interrupt::External => self.meie,
                Interrupt::Software => self.msie,
                Interrupt::Timer => self.timer_interrupt.mtie,
            }
    }

    fn interrupt_pending(&self, int: Interrupt) -> bool {
        match int {
            Interrupt::External => self.meip,
            Interrupt::Software => self.msip,
            Interrupt::Timer => self.timer_interrupt.mtip(),
        }
    }

    /// Evaluate the conditions for whether an interrupt should trap.
    /// If it should trap, store the current program counter in mepc,
    /// store the cause of the trap in mcause, write the current MIE
    /// bit (1) to the MPIE bit (p. 21 of the privileged spec), and
    /// return the address of an entry in the trap vector table;
    /// otherwise return None and do not modify mepc.
    fn interrupt_should_trap(&mut self, int: Interrupt, pc: u32) -> Option<u32> {
        if self.interrupt_enabled(int) && self.interrupt_pending(int) {
            self.save_mie_bit();
            self.mcause = Trap::Interrupt(int).mcause();
            self.mepc = pc;
            Some(self.trap_vector_address(Trap::Interrupt(int)))
        } else {
            None
        }
    }
}

/// M-mode machine state
///
/// Defines a simple RISC-V machine with only M-mode
/// and a minimal subset of the optional features.
///
/// This struct contains the core architectural state
/// of privileged mode, including the state of the
/// performance counters, interrupts, real time, etc.
#[derive(Debug, Default)]
pub struct Machine {
    /// Number of clock cycles since reset.
    mcycle: u64,
    /// Number of instructions executed since reset.
    minstret: u64,
    /// Machine trap scratch register
    pub mscratch: u32,
    /// Trap (interrupt and exception) control
    pub trap_ctrl: TrapCtrl,
}

impl Machine {
    pub fn csr_write_mcycle(&mut self, value: u32) {
        write_low_word(&mut self.mcycle, value);
    }

    pub fn csr_mcycle(&self) -> u32 {
        low_word(&self.mcycle)
    }

    pub fn csr_write_mcycleh(&mut self, value: u32) {
        write_high_word(&mut self.mcycle, value);
    }

    pub fn csr_mcycleh(&self) -> u32 {
        high_word(&self.mcycle)
    }

    pub fn csr_write_minstret(&mut self, value: u32) {
        write_low_word(&mut self.minstret, value);
    }

    pub fn csr_minstret(&self) -> u32 {
        low_word(&self.minstret)
    }

    pub fn csr_write_minstreth(&mut self, value: u32) {
        write_high_word(&mut self.mcycle, value);
    }

    pub fn csr_minstreth(&self) -> u32 {
        high_word(&self.minstret)
    }
}

fn low_word(value: &u64) -> u32 {
    (0xffff_ffff & value).try_into().unwrap()
}

fn write_low_word(target: &mut u64, value: u32) {
    let upper = *target & 0xffff_ffff_0000_0000;
    *target = upper | u64::from(value);
}

fn high_word(value: &u64) -> u32 {
    (0xffff_ffff & value >> 32).try_into().unwrap()
}

fn write_high_word(target: &mut u64, value: u32) {
    let lower = *target & 0xffff_ffff;
    *target = u64::from(value) << 32 | lower;
}
