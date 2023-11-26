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

use crate::{
    decode::Decoder,
    elf_utils::{ElfLoadError, ElfLoadable},
    utils::mask,
};

use self::{
    arch::{make_rv32i, make_rv32m},
    eei::Eei,
};

use super::{
    csr::MachineInterface,
    machine::Exception,
    memory::{Memory, Wordsize},
    pma::{
        PmaChecker, EXTINTCTRL_ADDR, MTIMECMPH_ADDR, MTIMECMP_ADDR,
        MTIMEH_ADDR, MTIME_ADDR, SOFTINTCTRL_ADDR, UARTTX_ADDR,
    },
    registers::Registers,
};

pub mod arch;
pub mod eei;
pub mod rv32i;
pub mod rv32m;

pub type ExecuteInstr<Eei> =
    fn(eei: &mut Eei, instr: u32) -> Result<(), Exception>;

#[derive(Debug, Default)]
pub struct Platform {
    registers: Registers,
    pma_checker: PmaChecker,
    memory: Memory,
    machine_interface: MachineInterface,
    decoder: Decoder<ExecuteInstr<Platform>>,
    pc: u32,
}

impl ElfLoadable for Platform {
    /// Write a byte to the EEPROM region of the platform. Returns an
    /// error on an attempt to write anything other than the eeprom region
    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), ElfLoadError> {
        if !self.pma_checker.in_eeprom(addr, 1) {
            Err(ElfLoadError::NonWritable(addr))
        } else {
            self.memory
                .write(addr.into(), data.into(), Wordsize::Byte)
                .expect("should work, address is 32-bit");
            Ok(())
        }
    }
}

impl Platform {
    /// Create the platform. Do not use Self::default(), which does
    /// not set up the decoder.
    pub fn new() -> Self {
        let mut decoder = Decoder::new(mask(7));
        make_rv32i(&mut decoder).expect("adding instructions should work");
        make_rv32m(&mut decoder).expect("adding instructions should work");

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
                self.pc = self.machine_interface
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
                self.pc = self.machine_interface
                    .machine
                    .trap_ctrl
                    .raise_exception(self.pc, Exception::IllegalInstruction);
                return;
            }
        };

        // Execute the instruction
        if let Err(ex) = executer(self, instr) {
            // If an exception occurred, raise it and return
            self.pc = self.machine_interface
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
        self.registers
            .write(x.into(), value.into())
            .expect("register index should be < 32, and value should be 32-bit")
    }

    fn x(&self, x: u8) -> u32 {
        self.registers
            .read(x.into())
            .expect("register index should be < 32")
            .try_into()
            .expect("register value should fit into u32")
    }

    fn increment_pc(&mut self) {
        self.pc = self.pc + 4
    }

    fn load(&self, addr: u32, width: Wordsize) -> Result<u32, Exception> {
        self.pma_checker.check_load(addr, width.width().into())?;
        // Match memory mapped registers first, then perform general load
        let result = match addr {
            MTIME_ADDR => self.machine_interface.machine.trap_ctrl.mmap_mtime(),
            MTIMEH_ADDR => {
                self.machine_interface.machine.trap_ctrl.mmap_mtimeh()
            }
            MTIMECMP_ADDR => {
                self.machine_interface.machine.trap_ctrl.mmap_mtimecmp()
            }
            MTIMECMPH_ADDR => {
                self.machine_interface.machine.trap_ctrl.mmap_mtimecmph()
            }
            SOFTINTCTRL_ADDR => todo!("implement load softintctrl"),
            EXTINTCTRL_ADDR => todo!("implement load extintctrl"),
            UARTTX_ADDR => todo!("implement load uarttx"),
            _ => self
                .memory
                .read(addr.into(), width)
                .expect("memory read should work")
                .try_into()
                .expect("value should fit into 32 bits"),
        };
        Ok(result)
    }

    fn store(
        &mut self,
        addr: u32,
        data: u32,
        width: Wordsize,
    ) -> Result<(), Exception> {
        self.pma_checker.check_store(addr, width.width().into())?;
        // Match memory mapped registers first, then perform general load
        match addr {
            MTIME_ADDR => self
                .machine_interface
                .machine
                .trap_ctrl
                .mmap_write_mtime(data),
            MTIMEH_ADDR => self
                .machine_interface
                .machine
                .trap_ctrl
                .mmap_write_mtimeh(data),
            MTIMECMP_ADDR => self
                .machine_interface
                .machine
                .trap_ctrl
                .mmap_write_mtimecmp(data),
            MTIMECMPH_ADDR => self
                .machine_interface
                .machine
                .trap_ctrl
                .mmap_write_mtimecmph(data),
            SOFTINTCTRL_ADDR => todo!("implement store softintctrl"),
            EXTINTCTRL_ADDR => todo!("implement store extintctrl"),
            UARTTX_ADDR => todo!("implement store uarttx"),
            _ => self
                .memory
                .write(addr.into(), data.into(), width)
                .expect("memory write should work"),
        };
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use crate::hart::machine::{MSTATUS_MIE, Trap};

    /// Simple wrapper to load 4 consecutive bytes
    fn write_instr(platform: &mut Platform, mut addr: u32, instr: u32) {
        for byte in instr.to_le_bytes().iter() {
            platform.write_byte(addr, *byte);
            addr += 1;
        }
    }

    #[test]
    fn check_state_on_reset() {
        let platform = Platform::new();
        let mstatus =
            platform.machine_interface.machine.trap_ctrl.csr_mstatus();
        let mie = mstatus >> MSTATUS_MIE & 1;

        assert_eq!(platform.pc, 0);
        assert_eq!(mie, 0);
        assert_eq!(
            platform.machine_interface.machine.trap_ctrl.csr_mcause(),
            0
        );
    }

    /// Load 0 at reset vector, execute, and expect jump to
    /// illegal instruction trap with mcause
    #[test]
    fn check_illegal_instruction_exception() {
        let mut platform = Platform::new();
        // Load an illegal instruction to reset vector
        write_instr(&mut platform, 0, 0);
        //println!("{platform:?}");
        // Attempt execution
        platform.step_clock();

        // Expect illegal instruction exception
        assert_eq!(platform.pc(), 0x0000_0008); // exception vector
	let mcause = platform.machine_interface.machine.trap_ctrl.csr_mcause();
	assert_eq!(mcause, Trap::Exception(Exception::IllegalInstruction).mcause())
    }
    
}
