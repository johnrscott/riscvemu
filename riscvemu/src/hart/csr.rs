//! Control and status registers
//!
//! From the unprivileged spec version 20191213, chapter 9: "RISC-V
//! defines a separate address space of 4096 Control and Status
//! registers associated with each hart". These registers are mainly
//! associated with various privileged mode operations.
//!

use crate::{extract_field, utils::extract_field};

use super::memory::Memory;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum CsrError {
    #[error("CSR 0x{0:x} does not exist (illegal instruction)")]
    NonExistentCsr(u16),
    #[error("Attempted write to read-only CSR 0x{0:x} (illegal instruction)")]
    ReadOnlyCsr(u16),
    #[error("CSR 0x{0:x} required higher privilege (illegal instruction)")]
    PrivilegedCsr(u16),
}

/// Is the CSR read-only?
fn read_only_csr(csr: u16) -> bool {
    extract_field(csr, 11, 10) == 0b11 
}

/// Control and status registers (CSR)
///
/// Implements CSRs as documented in chapter 2 or the privileged
/// spec (v20211203)
///
#[derive(Debug, Default)]
pub struct Csr {
    memory: Memory,
}

impl Csr {
    /// Read a value from a CSR
    pub fn read(&mut self, csr: u16, value: u32) -> Result<u32, CsrError> {
	Ok(0)
    }

    /// Write a value from a CSR
    pub fn write(&mut self, csr: u16, value: u32) -> Result<u32, CsrError> {
	if read_only_csr(csr) {
	    Err(CsrError::ReadOnlyCsr(csr))
	} else {
	    Ok(0)
	}
    }   
}


   
