#[macro_export]
macro_rules! define_branch_printer {
    ($instr_name:expr) => {
        fn printer(instr: u32) -> String {
            let SBtype {
                rs1: src1,
                rs2: src2,
                imm: offset,
            } = decode_btype(instr);
            format!("{} x{src1}, x{src2}, 0x{offset:x}", $instr_name)
        }
    };
}
pub use define_branch_printer;

#[macro_export]
macro_rules! define_load_printer {
    ($instr_name:expr) => {
        fn printer(instr: u32) -> String {
            let Itype {
                rs1: base,
                imm: offset,
                rd: dest,
            } = decode_itype(instr);
            format!("{} x{dest}, 0x{offset:x}(x{base})", $instr_name)
        }
    };
}
pub use define_load_printer;

#[macro_export]
macro_rules! define_store_printer {
    ($instr_name:expr) => {
        fn printer(instr: u32) -> String {
            let SBtype {
                rs1: base,
                rs2: src,
                imm: offset,
            } = decode_stype(instr);
            format!("{} x{src}, 0x{offset:x}(x{base})", $instr_name)
        }
    };
}
pub use define_store_printer;

#[macro_export]
macro_rules! define_reg_imm_printer {
    ($instr_name:expr) => {
        fn printer(instr: u32) -> String {
            let Itype {
                rs1: src,
                imm: i_immediate,
                rd: dest,
            } = decode_itype(instr);
            format!("{} x{dest}, x{src}, 0x{i_immediate:x}", $instr_name)
        }
    };
}
pub use define_reg_imm_printer;

#[macro_export]
macro_rules! define_reg_reg_printer {
    ($instr_name:expr) => {
        pub fn printer(instr: u32) -> String {
            let Rtype {
                rs1: src1,
                rs2: src2,
                rd: dest,
            } = decode_rtype(instr);
            format!("{} x{dest}, x{src1}, x{src2}", $instr_name)
        }
    };
}
pub use define_reg_reg_printer;

pub fn get_csr_name(addr: u16) -> String {
    match addr {
        _ => "unknown-csr",
    }
    .to_string()
}

#[macro_export]
macro_rules! define_csr_reg_printer {
    ($instr_name:expr) => {
        fn printer(instr: u32) -> String {
            use crate::hart::platform::print_macros::get_csr_name;
            let Itype {
                rs1: source,
                imm: csr,
                rd: dest,
            } = decode_itype(instr);
            let csr_name = get_csr_name(csr);
            format!("{} x{dest}, {csr_name}, x{source}", $instr_name)
        }
    };
}
pub use define_csr_reg_printer;

#[macro_export]
macro_rules! define_csr_imm_printer {
    ($instr_name:expr) => {
        pub fn printer(instr: u32) -> String {
            use crate::hart::platform::print_macros::get_csr_name;
            let Itype {
                rs1: uimm,
                imm: csr,
                rd: dest,
            } = decode_itype(instr);
            let csr_name = get_csr_name(csr);
            format!("{} x{dest}, {csr_name}, 0x{uimm:x}", $instr_name)
        }
    };
}
pub use define_csr_imm_printer;
