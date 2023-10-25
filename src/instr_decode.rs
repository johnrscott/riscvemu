/// In the fields below, references to registers
/// provide the index of the register, not its value.
enum Instr {
    /// Load imm into the high 20 bits of rd
    Lui { rd: u8, imm: u32 },
    /// Load imm into the high 20 bits of the pc
    Auipc { rd: u8, imm: u32 },
    /// Store the current pc+4 in rd, and set
    /// pc = pc + imm, where imm is a multiple of 2.
    Jal { rd: u8, imm: u32 },
    /// Store the current pc+4 in rd, and set
    /// pc = rs1 + imm (imm is a multiple of 2)
    Jalr { rd: u8, rs1: u8, imm: u32 },
    /// If rs1 == rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Beq { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 != rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Bne { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 < rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Blt { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 >= rs2, set pc = pc + imm, where
    /// imm is a multiple of two; else do nothing.
    Bge { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 < rs2, set pc = pc + imm, where
    /// imm is a multiple of two, treating the
    /// contents of rs1 and rs2 as unsigned;
    /// else do nothing.
    Bltu { rs1: u8, rs2: u8, imm: u32 },
    /// If rs1 >= rs2, set pc = pc + imm, where
    /// imm is a multiple of two, treating the
    /// contents of rs1 and rs2 as unsigned;
    /// else do nothing.
    Bltu { rs1: u8, rs2: u8, imm: u32 },
    /// Load the byte at address rs1 + imm into rd
    Lb { rd: u8, rs1: u8, imm: u32 },
    /// Load the halfword at address rs1 + imm into rd
    Lh { rd: u8, rs1: u8, imm: u32 },
    /// Load the word at address rs1 + imm into rd
    Lw { rd: u8, rs1: u8, imm: u32 },
    /// Store the byte in rs1 to address rs1 + imm
    Sb { rs1: u8, rs2: u8, imm: u32 },
    /// Store the halfword in rs1 to address rs1 + imm
    Sh { rs1: u8, rs2: u8, imm: u32 },
    /// Store the word in rs1 to address rs1 + imm
    Sw { rs1: u8, rs2: u8, imm: u32 },

    
    
}
