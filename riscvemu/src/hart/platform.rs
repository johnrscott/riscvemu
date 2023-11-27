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
        let instr = match self.fetch_instruction() {
            Ok(instr) => instr,
            Err(ex) => {
                // On exception during exception fetch, raise it and return
                self.pc = self
                    .machine_interface
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
                self.pc = self
                    .machine_interface
                    .machine
                    .trap_ctrl
                    .raise_exception(self.pc, Exception::IllegalInstruction);
                return;
            }
        };

        // Execute the instruction
        if let Err(ex) = executer(self, instr) {
            // If an exception occurred, raise it and return
            self.pc = self
                .machine_interface
                .machine
                .trap_ctrl
                .raise_exception(self.pc, ex);
            return;
        }

        // If instruction completed successfully, increment count
        // of retired instructions
        self.machine_interface.machine.increment_minstret();
    }

    fn fetch_instruction(&self) -> Result<u32, Exception> {
        self.pma_checker.check_instruction_fetch(self.pc)?;
        let instr = self
            .memory
            .read(self.pc.into(), Wordsize::Word)
            .expect("read should succeed ")
            .try_into()
            .expect("result should fit in 32 bits");
        Ok(instr)
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
mod tests {

    use super::*;
    use crate::encode::*;
    use crate::hart::machine::{Trap, MSTATUS_MIE};

    /// Simple wrapper to load 4 consecutive bytes
    fn write_instr(platform: &mut Platform, mut addr: u32, instr: u32) {
        for byte in instr.to_le_bytes().iter() {
            platform
                .write_byte(addr, *byte)
                .expect("writing instruction should work; fix address if not");
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

        // Attempt execution
        platform.step_clock();

        // Expect illegal instruction exception
        assert_eq!(platform.pc(), 0x0000_0008); // exception vector
        let mcause = platform.machine_interface.machine.trap_ctrl.csr_mcause();
        assert_eq!(
            mcause,
            Trap::Exception(Exception::IllegalInstruction).mcause()
        )
    }

    #[test]
    fn check_lui() -> Result<(), &'static str> {
        // Check a basic case of lui (result should be placed in
        // upper 20 bits of x2)
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, lui!(x2, 53));
        platform.step_clock();
        let x2 = platform.x(2);
        assert_eq!(x2, 53 << 12);
        assert_eq!(platform.pc(), 4);
        Ok(())
    }

    #[test]
    fn check_auipc() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        platform.set_pc(8);
        write_instr(&mut platform, 8, auipc!(x4, 53));
        platform.step_clock();
        let x4 = platform.x(4);
        assert_eq!(x4, 8 + (53 << 12));
        assert_eq!(platform.pc(), 12);
        Ok(())
    }

    #[test]
    fn check_jalr() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        platform.set_pc(12);
        platform.set_x(6, 20);
        write_instr(&mut platform, 12, jalr!(x4, x6, -4));
        platform.step_clock();
        let x4 = platform.x(4);
        assert_eq!(x4, 16);
        assert_eq!(platform.pc(), 20 - 4);
        Ok(())
    }

    #[test]
    fn check_beq_not_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, beq!(x1, x2, 16));
        platform.set_x(1, 1);
        platform.set_x(2, 2);
        platform.step_clock();
        assert_eq!(platform.pc(), 4);
        Ok(())
    }

    #[test]
    fn check_beq_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, beq!(x1, x2, 16));
        platform.set_x(1, 2);
        platform.set_x(2, 2);
        platform.step_clock();
        assert_eq!(platform.pc(), 16);
        Ok(())
    }

    #[test]
    fn check_bne_not_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bne!(x1, x2, 16));
        platform.set_x(1, 2);
        platform.set_x(2, 2);
        platform.step_clock();
        assert_eq!(platform.pc(), 4);
        Ok(())
    }

    #[test]
    fn check_bne_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bne!(x1, x2, 16));
        platform.set_x(1, 1);
        platform.set_x(2, 2);
        platform.step_clock();
        assert_eq!(platform.pc(), 16);
        Ok(())
    }

    #[test]
    fn check_blt_not_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, blt!(x1, x2, 16));
        platform.set_x(1, 10);
        platform.set_x(2, 0xffff_ffff);
        platform.step_clock();
        assert_eq!(platform.pc(), 4);
        Ok(())
    }

    #[test]
    fn check_blt_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, blt!(x1, x2, 16));
        platform.set_x(1, 0xffff_ffff);
        platform.set_x(2, 10);
        platform.step_clock();
        assert_eq!(platform.pc(), 16);
        Ok(())
    }

    #[test]
    fn check_bltu_not_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bltu!(x1, x2, 16));
        platform.set_x(1, 10);
        platform.set_x(2, 1);
        platform.step_clock();
        assert_eq!(platform.pc(), 4);
        Ok(())
    }

    #[test]
    fn check_bltu_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bltu!(x1, x2, 16));
        platform.set_x(1, 1);
        platform.set_x(2, 10);
        platform.step_clock();
        assert_eq!(platform.pc(), 16);
        Ok(())
    }

    #[test]
    fn check_bge_not_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bge!(x1, x2, 16));
        platform.set_x(1, 0xffff_ffff);
        platform.set_x(2, 10);
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_bge_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bge!(x1, x2, 16));
        platform.set_x(1, 10);
        platform.set_x(2, 0xffff_ffff);
        platform.step_clock();
        assert_eq!(platform.pc, 16);
        Ok(())
    }

    #[test]
    fn check_bgeu_not_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bgeu!(x1, x2, 16));
        platform.set_x(1, 1);
        platform.set_x(2, 10);
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_bgeu_taken() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, bgeu!(x1, x2, 16));
        platform.set_x(1, 10);
        platform.set_x(2, 1);
        platform.step_clock();
        assert_eq!(platform.pc, 16);
        Ok(())
    }

    #[test]
    fn check_lb() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, lb!(x1, x2, 16));
        platform.set_x(2, 0x0002_0000);
        let addr = 0x0002_0010; // Ensure in main memory
        platform.store(addr, 0xff, Wordsize::Byte).unwrap();
        platform.step_clock();
        //assert_eq!(platform.pc(), 4);
        assert_eq!(platform.x(1), 0xffff_ffff);
        Ok(())
    }

    #[test]
    fn check_lbu() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, lbu!(x1, x2, 16));
        platform.set_x(2, 0x0002_0000);
        let addr = 0x0002_0010; // Ensure in main memory
        platform.store(addr, 0xff, Wordsize::Byte).unwrap();
        platform.step_clock();
        assert_eq!(platform.pc(), 4);
        assert_eq!(platform.x(1), 0x0000_00ff);
        Ok(())
    }

    #[test]
    fn check_lh() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, lh!(x1, x2, 16));
        platform.set_x(2, 5);
        platform.store(21, 0xff92, Wordsize::Halfword).unwrap();
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        assert_eq!(platform.x(1), 0xffff_ff92);
        Ok(())
    }

    #[test]
    fn check_lhu() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, lhu!(x1, x2, 16));
        platform.set_x(2, 5);
        platform.store(21, 0xff92, Wordsize::Halfword).unwrap();
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        assert_eq!(platform.x(1), 0x0000_ff92);
        Ok(())
    }

    #[test]
    fn check_lw() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, lw!(x1, x2, 16));
        platform.set_x(2, 6);
        platform.store(22, 0x1234_ff92, Wordsize::Word).unwrap();
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        assert_eq!(platform.x(1), 0x1234_ff92);
        Ok(())
    }

    #[test]
    fn check_sb() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sb!(x1, x2, 16));
        platform.set_x(1, 0xfe);
        platform.set_x(2, 6);
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        assert_eq!(platform.load(22, Wordsize::Byte).unwrap(), 0xfe);
        Ok(())
    }

    #[test]
    fn check_sh() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sh!(x1, x2, 16));
        platform.set_x(1, 0xabfe);
        platform.set_x(2, 7);
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        assert_eq!(platform.load(23, Wordsize::Halfword).unwrap(), 0xabfe);
        Ok(())
    }

    #[test]
    fn check_sw() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sw!(x1, x2, -15));
        platform.set_x(1, 0xabcd_ef12);
        platform.set_x(2, 20);
        platform.step_clock();
        assert_eq!(platform.pc, 4);
        assert_eq!(platform.load(5, Wordsize::Word).unwrap(), 0xabcd_ef12);
        Ok(())
    }

    #[test]
    fn check_addi() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, addi!(x1, x2, -23));
        platform.set_x(2, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xffff_ffff);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_slti_both_positive() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slti!(x1, x2, 22));
        platform.set_x(2, 124);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slti!(x1, x2, 124));
        platform.set_x(2, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slti_both_negative() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slti!(x1, x2, -5));
        let v1: u32 = interpret_i32_as_unsigned!(-24).into();
        let v2: u32 = interpret_i32_as_unsigned!(-5).into();
        platform.set_x(2, v1);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slti!(x1, x2, -24));
        platform.set_x(2, v2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slti_different_signs() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slti!(x1, x2, 5));
        let v1: u32 = interpret_i32_as_unsigned!(-24).into();
        let v2: u32 = 5;
        platform.set_x(2, v1);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slti!(x1, x2, -24));
        platform.set_x(2, v2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_sltui() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sltiu!(x1, x2, 22));
        platform.set_x(2, 124);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        platform
            .memory
            .write(0, sltiu!(x1, x2, 124).into(), Wordsize::Word);
        platform.set_x(2, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_andi() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, andi!(x1, x2, 0xff0));
        platform.set_x(2, 0x00ff_ff00);
        platform.step_clock();
        let x1 = platform.x(1);
        // Note that AND uses the sign-extended 12-bit immediate
        assert_eq!(x1, 0x00ff_ff00);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_ori() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, ori!(x1, x2, 0xff0));
        platform.set_x(2, 0x00ff_ff00);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xffff_fff0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_xori() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, xori!(x1, x2, 0xff0));
        platform.set_x(2, 0x00ff_ff00);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xff00_00f0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_slli() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slli!(x1, x2, 2));
        platform.set_x(2, 0b1101);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0b110100);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_srli() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, srli!(x1, x2, 4));
        platform.set_x(2, 0xf000_0f00);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0x0f00_00f0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_srai() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, srai!(x1, x2, 4));
        platform.set_x(2, 0xf000_0f00);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xff00_00f0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_add() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, add!(x1, x2, x3));
        platform.set_x(2, 2);
        platform.set_x(3, 3);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 5);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_add_wrapping_edge_case() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, add!(x1, x2, x3));
        platform.set_x(2, 0xffff_fffe);
        platform.set_x(3, 5);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 3);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sub() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sub!(x1, x2, x3));
        platform.set_x(2, 124);
        platform.set_x(3, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 102);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sub_wrapping_edge_case() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sub!(x1, x2, x3));
        platform.set_x(2, 20);
        platform.set_x(3, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xffff_fffe);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_slt_both_positive() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slt!(x1, x2, x3));
        platform.set_x(2, 124);
        platform.set_x(3, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slt!(x1, x2, x3));
        platform.set_x(3, 124);
        platform.set_x(2, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slt_both_negative() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slt!(x1, x2, x3));
        let v1: u32 = interpret_i32_as_unsigned!(-24).into();
        let v2: u32 = interpret_i32_as_unsigned!(-5).into();
        platform.set_x(2, v1);
        platform.set_x(3, v2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slt!(x1, x2, x3));
        platform.set_x(3, v1);
        platform.set_x(2, v2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_slt_different_signs() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slt!(x1, x2, x3));
        let v1: u32 = interpret_i32_as_unsigned!(-24).into();
        let v2: u32 = 5;
        platform.set_x(2, v1);
        platform.set_x(3, v2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, slt!(x1, x2, x3));
        platform.set_x(3, v1);
        platform.set_x(2, v2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_sltu() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sltu!(x1, x2, x3));
        platform.set_x(2, 124);
        platform.set_x(3, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0);
        assert_eq!(platform.pc, 4);

        // Swap src1 and src2
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sltu!(x1, x2, x3));
        platform.set_x(3, 124);
        platform.set_x(2, 22);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);

        Ok(())
    }

    #[test]
    fn check_and() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, and!(x1, x2, x3));
        platform.set_x(2, 0x00ff_ff00);
        platform.set_x(3, 0x0f0f_f0f0);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0x000f_f000);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_or() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, or!(x1, x2, x3));
        platform.set_x(2, 0x00ff_ff00);
        platform.set_x(3, 0x0f0f_f0f0);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0x0fff_fff0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_xor() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, xor!(x1, x2, x3));
        platform.set_x(2, 0x00ff_ff00);
        platform.set_x(3, 0x0f0f_f0f0);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0x0ff0_0ff0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sll() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sll!(x1, x2, x3));
        platform.set_x(2, 0b1101);
        platform.set_x(3, 2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0b110100);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_srl() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, srl!(x1, x2, x3));
        platform.set_x(2, 0xf000_0f00);
        platform.set_x(3, 4);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0x0f00_00f0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_sra() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, sra!(x1, x2, x3));
        platform.set_x(2, 0xf000_0f00);
        platform.set_x(3, 4);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xff00_00f0);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mul() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, mul!(x1, x2, x3));
        platform.set_x(2, 5);
        platform.set_x(3, interpret_i32_as_unsigned!(-4).into());
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, interpret_i32_as_unsigned!(-20).into());
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulh_positive() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, mulh!(x1, x2, x3));
        platform.set_x(2, 0x7fff_ffff);
        platform.set_x(3, 4);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulh_negative() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, mulh!(x1, x2, x3));
        platform.set_x(2, 0xffff_ffff);
        platform.set_x(3, 4);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xffff_ffff);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulhu() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, mulhu!(x1, x2, x3));
        platform.set_x(2, 0xffff_ffff);
        platform.set_x(3, 4);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 3);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulhsu_1() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, mulhsu!(x1, x2, x3));
        platform.set_x(2, 0xffff_ffff);
        platform.set_x(3, 4);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0xffff_ffff);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_mulhsu_2() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, mulhsu!(x1, x2, x3));
        platform.set_x(2, 4);
        platform.set_x(3, 0xffff_ffff);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 3);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_div() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, div!(x1, x2, x3));
        platform.set_x(2, 6);
        platform
            .registers
            .write(3, interpret_i32_as_unsigned!(-3).into());
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, interpret_i32_as_unsigned!(-2).into());
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_div_round_towards_zero() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, div!(x1, x2, x3));
        platform.set_x(2, 10);
        platform
            .registers
            .write(3, interpret_i32_as_unsigned!(-3).into());
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, interpret_i32_as_unsigned!(-3).into());
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_divu() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, divu!(x1, x2, x3));
        platform.set_x(2, 0xe000_0000);
        platform.set_x(3, 2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 0x7000_0000);
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_rem() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, rem!(x1, x2, x3));
        platform
            .registers
            .write(2, interpret_i32_as_unsigned!(-10).into());
        platform.set_x(3, 3);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, interpret_i32_as_unsigned!(-1).into());
        assert_eq!(platform.pc, 4);
        Ok(())
    }

    #[test]
    fn check_remu() -> Result<(), &'static str> {
        let mut platform = Platform::new();
        write_instr(&mut platform, 0, remu!(x1, x2, x3));
        platform.set_x(2, 0xe000_0003);
        platform.set_x(3, 2);
        platform.step_clock();
        let x1 = platform.x(1);
        assert_eq!(x1, 1);
        assert_eq!(platform.pc, 4);
        Ok(())
    }
}
