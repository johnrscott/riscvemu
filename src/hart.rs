use memory::Memory;

pub mod memory;

/// RISC-V Hardware Thread
///
/// This is the simplest possible RISC-V hardware
/// thread, which is an execution environment interface
/// where (see section 1.2 in the specification):
///
/// * there is only one hart (this one), which supports
///   only a single privilege level (i.e. there is not
///   notion of privilege)
/// * the hart implements only RV32I
/// * the initial state of the program is defined by a
///   set of values of memory and registers (including
///   the program counter), determined as part of making
///   this object.
/// * all memory is readable and writable, and the full
///   address space is main memory (section 1.4)
/// * All required traps are fatal traps (section 1.6),
///   causing this execution environment (i.e. this single
///   hart) to terminate.
///
/// The member function step() controls execution of the hart.
/// Each time it is called, the instruction at the current PC
/// is executed. If an exception occurs, step() returns the
/// trap type that occurred, for the caller to take any action.
/// If step is re-called, then the hart will attempt to continue
/// execution, which may or may not result in another trap.
///
///
#[derive(Debug)]
struct Hart {
    memory: Memory,
    pc: u32,
    //registers: Registers,
}

// impl Hart {
//     pub fn new() -> Self {

//     }

// }
