//! RISC-V Platform
//!
//! This files contains a basic RISC-V platform that models a 32-bit
//! microcontroller. It supports only M-mode, implements the rv32im
//! architecture, and implements a minimal set of the required
//! privileged specification (e.g. many CSR registers that can be
//! read-only zero are implemented as read-only zero). The memory
//! models includes two devices: an EEPROM (non-volatile) for storing
//! instructions, and a RAM device for use during execution. Both are
//! 8 KiB. The device includes one peripheral: a virtual UART output
//! device, memory-mapped in an I/O region of the address
//! space. Writing a character to this UARTs register sends output to
//! the virtual UART bus, which can be read using an external
//! interface (modelling an debug connection to the microcontroller).
//!
//! See the pma module for documentation on the memory map. See the
//! csr module for documentation on the implemented control and status
//! registers.
//!
//! Programming the device is modelled by writing initial values into
//! the EEPROM memory region. The state of the platform is initialised
//! by a power-on reset. Progress is made by single stepping through
//! clock rising edges.
//!
//! Interrupt and exception traps are modelled. The software compiled
//! for this platform must write values to the trap vector table (part
//! of the EEPROM memory map.

use crate::{decode::Decoder, utils::mask};

use self::{eei::Eei, arch::make_rv32i};

use super::{
    csr::MachineInterface, machine::Exception, memory::Memory, pma::PmaChecker,
    registers::Registers,
};

pub mod arch;
pub mod eei;
pub mod exec;

pub type ExecuteInstr<Eei> = fn(eei: &mut Eei, instr: u32);

#[derive(Debug, Default)]
pub struct Platform {
    registers: Registers,
    pma_checker: PmaChecker,
    memory: Memory,
    machine_interface: MachineInterface,
    decoder: Decoder<ExecuteInstr<Platform>>,
    pc: u32,
}

impl Platform {
    /// Create the platform. Do not use Self::default(), which does
    /// not set up the decoder.
    pub fn new() -> Self {
        let mut decoder = Decoder::new(mask(7));
	make_rv32i(&mut decoder).expect("adding instructions should work");
        Self {
            decoder,
            ..Self::default()
        }
    }

    /// Reset the state of the platform. Reset is described in
    /// the privileged spec section 3.4. For this platform:
    ///
    /// * the mstatus field MIE is set to 0
    /// * the pc is set to the reset vector (0)
    /// * the mcause register is set 0 to indicate unspecified reset cause
    ///
    pub fn reset(&mut self) {}

    /// Single clock cycle step
    ///
    /// On the rising edge of the clock, perform the sequence of
    /// actions specified below. If a exception is raised during a
    /// step, then return without performing subsequent steps (todo:
    /// check whether this is valid with respect to instructions that
    /// can trigger multiple exceptions).
    ///
    /// * increment mcycle and mtime
    /// * fetch the instruction located at pc (can raise exception)
    /// * execute the instruction that was fetched (can raise exception)
    /// * increment minstret (i.e. only if instruction was completed)
    ///
    pub fn step_clock() {}
}

/// Implementation of the unprivileged execution environment interface
impl Eei for Platform {
    fn raise_exception(&mut self, ex: Exception) {
        unimplemented!("todo")
    }

    fn set_x(&mut self, x: u8, value: u32) {
	unimplemented!("todo")
    }

    fn increment_pc(&mut self) {
	self.pc = self.pc + 4
    }
}
