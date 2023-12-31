use crate::platform::machine::Exception;

use super::{eei::Eei, Instr};

pub fn mret<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(eei: &mut E, _instr: u32) -> Result<(), Exception> {
        eei.mret();
        Ok(())
    }

    fn printer(_instr: u32) -> String {
        "mret".to_string()
    }

    Instr { executer, printer }
}

pub fn ecall<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(_eei: &mut E, _instr: u32) -> Result<(), Exception> {
        Err(Exception::MmodeEcall)
    }

    fn printer(_instr: u32) -> String {
        "ecall".to_string()
    }

    Instr { executer, printer }
}

pub fn ebreak<E: Eei>() -> Instr<E> {
    fn executer<E: Eei>(_eei: &mut E, _instr: u32) -> Result<(), Exception> {
        Err(Exception::Breakpoint)
    }

    fn printer(_instr: u32) -> String {
        "ebreak".to_string()
    }

    Instr { executer, printer }
}
