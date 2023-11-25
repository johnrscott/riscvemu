//! # Physical Memory Attributes
//!
//! This file defines the physical memory layout and attributes of the
//! RISC-V processor.
//!
//! ## Memory Map
//!
//! The memory map for the processor is as follows. Address ranges
//! are listed in the format A-B, where address A is the first byte
//! of the region and address B is the first byte above the region.
//!
//! ### Vector table (0x0000_0000-0x0000_0088)
//!
//! The following four-byte words in this region are used.
//! Other addresses hold the value zero. The interrupt vector
//! table reserves space for the full set of 32 interrupts.
//!
//! | Address | Width | Descrption |
//! |---------|-------|------------|
//! | 0x0000_0000 | 4 | Reset vector (pc points here on reset) |
//! | 0x0000_0004 | 4 | Non-maskable interrupt vector |
//! | 0x0000_0008 | 4 | Trap vector table base (exception vector) |
//! | 0x0000_0014 | 4 | Machine software interrupt vector |
//! | 0x0000_0024 | 4 | Machine timer interrupt vector |
//! | 0x0000_0034 | 4 | Machine external interrupt vector |
//!
//!
//! ### Memory-mapped machine timer registers (0x0000_0088-0x0000_0098)
//!
//! | Address | Width | Description |
//! |---------|-------|-------------|
//! | 0x0000_0088 | 8 | mtime (64-bit real time) |
//! | 0x0000_0090 | 8 | mtimecmp (64-bit timer compare register) |
//!
//! ### Input/Output memory region (0x0000_0098-0x0000_0200)
//!
//! Input/output device memory mappings
//!
//! ### 16 KiB Main memory (0x0000_0200-0x0000_4200)
//!
//! 16 KiB main memory (read/write/execute). Contains all code and data.
//! 
