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

