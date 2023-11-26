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

use crate::hart::machine::Exception;

/// Execution environment interface
pub trait Eei {
    /// Raise an exception
    ///
    /// This will cause a transfer of control to the exception trap
    /// handler in the execution environment. After the exception has
    /// been handled, control will return to the instruction that
    /// triggered the exception.
    fn raise_exception(&mut self, ex: Exception);

    /// Set the register x to value.
    ///
    /// Panic if the register index x is out of range (x >
    /// 32). Writing to register 0 has no effect.
    fn set_x(&mut self, x: u8, value: u32);

    /// Set pc = pc + 4
    fn increment_pc(&mut self);
}
