//! M-mode Implementation
//!
//! This file contains the implementation of a basic M-mode-only
//! RISC-V machine.
//!
//! References to the privileged spec refer to version 20211203.
//! 

use thiserror::Error;

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
    External
}

#[derive(Copy, Clone)]
pub enum Trap {
    Interrupt(Interrupt),
    Exception(Exception),
}

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
	    Self::Interrupt(int) => {
		match int {
		    Interrupt::Software => 3,
		    Interrupt::Timer => 7,
		    Interrupt::External => 11,
		}
	    }
	    Self::Exception(ex) => {
		match ex {
		    Exception::InstructionAddressMisaligned => 0,
		    Exception::InstructionAccessFault => 1,
		    Exception::IllegalInstruction => 2,
		    Exception::Breakpoint => 3,
		    Exception::LoadAddressMisaligned => 4,
		    Exception::LoadAccessFault => 5,
		    Exception::StoreAddressMisaligned => 6,
		    Exception::StoreAccessFault => 7,
		    Exception::MmodeEcall => 11,
		}
	    }
	    
	}
    }
    
}

#[derive(Debug, Error)]
pub enum TrapCtrlError {
    #[error("Trap vector base address must be four byte aligned")]
    TrapVectorBaseMisaligned
}

/// Trap control
///
/// This implementation uses 
#[derive(Debug, Default)]
pub struct TrapCtrl {
    /// Global interrupt enable bit in mstatus (MIE)
    mstatus_mie: bool,
    /// Machine interrupt enable
    mie: u32,
    /// Machine interrupt pending
    mip: u32,
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
}

pub const MTVEC_MODE_VECTORED: u32 = 1;

impl TrapCtrl {

    /// In this implementation, the mtvec CSR is read-only
    pub fn mtvec(&self) -> u32 {
	(self.trap_vector_base << 2) | MTVEC_MODE_VECTORED
    }

    /// For an exception, return the trap vector base address. For an
    /// interrupt, return a trap vector based on the cause.
    pub fn trap_vector_address(&self, trap: Trap) -> u32 {
	match &trap {
	    Trap::Exception(_) => self.trap_vector_base,
	    Trap::Interrupt(int) => self.trap_vector_base + 4*trap.cause()
	}
    }
    
    /// Make a new trap control struct
    ///
    /// Returns an error if the trap_vector_base address is not
    /// four-byte aligned
    fn new(trap_vector_base: u32) -> Result<Self, TrapCtrlError> {
	if trap_vector_base % 4 != 0 {
	    Err(TrapCtrlError::TrapVectorBaseMisaligned)
	} else {
	    Ok(Self {
		trap_vector_base, ..Self::default()
	    })
	}
    }
    
    /// Set the interrupt pending flag for the interrupt in mip
    ///
    /// This is the first step in raising an interrupt. This route
    /// to raising an interrupt is used by hardware. Software can
    /// make an interrupt pending by writing to the mip through the
    /// CSR.
    ///
    /// Once an interrupt is pending, it will cause an interrupt
    /// in a bounded amount of time if other conditions 
    pub fn set_interrupt_pending(&mut self, int: Interrupt) {

	let bit_position = Trap::Interrupt(int).cause();
	self.mip |= 1 << bit_position;
    }

    /// Evaluate the conditions for whether an interrupt should trap.
    /// If it should trap, store the current program counter in mepc,
    /// store the cause of the trap in mcause, and return the address
    /// of an entry in the trap vector table; otherwise return None
    /// and do not modify mepc.
    fn interrupt_should_trap(&mut self, int: Interrupt, pc: u32) -> Option<u32> {
	if self.interrupt_enabled(int) && self.interrupt_pending(int) {
	    self.mcause = Trap::Interrupt(int).mcause();
	    self.mepc = pc;
	    Some(self.trap_vector_address(Trap::Interrupt(int)))
	} else {
	    None
	}
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
    /// todo...
    pub fn mret(&mut self) {

    }
    
    fn interrupts_globally_enabled(&self) -> bool {
	self.mstatus_mie
    }
    
    fn interrupt_enabled(&self, int: Interrupt) -> bool {
	let bit_position = Trap::Interrupt(int).cause();
	let interrupt_enabled = self.mie & (1 << bit_position) != 0;
	self.interrupts_globally_enabled() && interrupt_enabled
    }

    fn interrupt_pending(&self, int: Interrupt) -> bool {
	let bit_position = Trap::Interrupt(int).cause();
	self.mip & (1 << bit_position) != 0
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
    /// Real time
    mtime: u64,
    /// Trap (interrupt and exception) control
    trap_ctrl: TrapCtrl,
}
