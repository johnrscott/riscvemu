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

use self::{arch::make_rv32i, eei::Eei};

use super::{
    csr::MachineInterface,
    machine::Exception,
    memory::{Memory, Wordsize},
    pma::PmaChecker,
    registers::Registers,
};

pub mod arch;
pub mod eei;
pub mod rv32i;

pub type ExecuteInstr<Eei> = fn(eei: &mut Eei, instr: u32) -> Result<(), Exception>;

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
    /// actions specified below. If a trap (interrupt or exception) is
    /// encountered during a step, then return without performing
    /// subsequent steps (todo: check whether this is valid with
    /// respect to instructions that can trigger multiple exceptions).
    ///
    /// * increment mcycle and mtime
    /// * check for pending interrupts. If pending, return early
    /// * fetch the instruction located at pc (can raise exception)
    /// * execute the instruction that was fetched (can raise exception)
    /// * increment minstret (i.e. only if instruction was completed)
    ///
    pub fn step_clock(&mut self) {
        // Increment machine counters
        self.machine_interface.machine.increment_mcycle();
        self.machine_interface.machine.trap_ctrl.increment_mtime();

        // Check for pending interrupts. If an interrupt is pending,
        // set the pc to the interrupt handler vector and return.
        if let Some(interrupt_pc) = self
            .machine_interface
            .machine
            .trap_ctrl
            .trap_interrupt(self.pc)
        {
            self.pc = interrupt_pc;
            return;
        }

        // Fetch the instruction at the current pc.
        let instr = match self.load(self.pc, Wordsize::Word) {
            Ok(instr) => instr,
            Err(ex) => {
                // On exception during exception fetch, raise it and return
                self.machine_interface
                    .machine
                    .trap_ctrl
                    .raise_exception(self.pc, ex);
                return;
            }
        };

        // Decode the instruction
        let executer = match self.decoder.get_exec(instr) {
            Ok(executer) => executer,
            Err(_) => {
                // If instruction is not decoded successfully, return
                // illegal instruction
                self.machine_interface
                    .machine
                    .trap_ctrl
                    .raise_exception(self.pc, Exception::IllegalInstruction);
                return;
            }
        };

        // Execute the instruction
        if let Err(ex) = executer(self, instr) {
            // If an exception occurred, raise it and return
            self.machine_interface
                .machine
                .trap_ctrl
                .raise_exception(self.pc, ex);
            return;
        }

        // If instruction completed successfully, increment count
        // of retired instructions
        self.machine_interface.machine.increment_minstret();
    }
}

/// Implementation of the unprivileged execution environment interface
impl Eei for Platform {
    fn set_pc(&mut self, pc: u32) {
        self.pc = pc;
    }

    fn pc(&self) -> u32 {
        self.pc
    }

    fn set_x(&mut self, x: u8, value: u32) {
        unimplemented!("todo")
    }

    fn x(&self, x: u8) -> u32 {
        unimplemented!("todo")
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc + 4
    }

    fn load(&self, addr: u32, width: Wordsize) -> Result<u32, Exception> {
        unimplemented!("todo")
    }

    fn store(&self, addr: u32, data: u32, width: Wordsize) -> Result<(), Exception> {
        unimplemented!("todo")
    }
}
