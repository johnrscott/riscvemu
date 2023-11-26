//! Unprivileged Execution Environment Interface
//!
//! This file defines a trait Eei which is the execution environment
//! interface (EEI) for the unprivileged-mode software that runs on
//! the RISC-V platform.
//!
//! References to the RISC-V unprivileged specification use version
//! 20191213.
//!
//! The EEI is the environment that a RISC-V program "sees" when it is
//! executing on the platform (see section 1.2 unprivileged
//! spec). Here, that is interpreted as an interface that can be used
//! to implement the user-mode architecture (including the
//! instructions of the instruction sets which are implemented). This
//! behaviour of the implementation is mostly specified in the
//! unprivileged RISC-V specification.
//!
//! The implementation of the EEI itself is described in the
//! privileged mode specification, and includes the implementation of,
//! traps, physical memory protection, control and status registers,
//! and other privileged-mode behaviour.
//!
//! In this program, the EEI is a trait. Implementing this trait means
//! specifying the implementation of the privileged
//! architecture. Unprivileged architecture is implemented in terms of
//! the behaviour exposed by the EEI trait.

use crate::hart::{machine::Exception, memory::Wordsize};

/// Execution environment interface
pub trait Eei {
    /// Set the program counter
    fn set_pc(&mut self, pc: u32);

    /// Get the current program counter
    fn pc(&self) -> u32;

    /// Set the register x to value.
    ///
    /// Panic if the register index x is out of range (x >
    /// 32). Writing to register 0 has no effect.
    fn set_x(&mut self, x: u8, value: u32);

    /// Get the value of register with index x
    ///
    /// Panic if the register x is out of range (x > 32). Reading
    /// register 0 always returns 0.
    fn x(&self, x: u8) -> u32;

    /// Set pc = pc + 4
    fn increment_pc(&mut self);

    /// Load a value from memory
    ///
    /// The address and width are checked using the physical memory
    /// attributes (PMA) checker, which can return an
    /// exception. Otherwise, the result of the load is returned.
    fn load(&self, addr: u32, width: Wordsize) -> Result<u32, Exception>;

    /// Store a value to memory
    ///
    /// The address and width are checked using the physical memory
    /// attributes (PMA) checker, which can return an
    /// exception. Otherwise, the data is stored to memory.
    fn store(
        &mut self,
        addr: u32,
        data: u32,
        width: Wordsize,
    ) -> Result<(), Exception>;
}
