pub const OP_LUI: u32 = 0b0110111;
pub const OP_AUIPC: u32 = 0b0010111;
pub const OP_JAL: u32 = 0b1101111;
pub const OP_JALR: u32 = 0b1100111;
pub const OP_IMM: u32 = 0b0010011;
pub const OP_IMM_32: u32 = 0b0011011;
pub const OP: u32 = 0b0110011;
pub const OP_32: u32 = 0b0111011;
pub const OP_BRANCH: u32 = 0b1100011;
pub const OP_LOAD: u32 = 0b0000011;
pub const OP_STORE: u32 = 0b0100011;
pub const OP_MISC_MEM: u32 = 0b0001111;
pub const OP_SYSTEM: u32 = 0b1110011;

// Conditional branches
pub const FUNCT3_BEQ: u32 = 0b000;
pub const FUNCT3_BNE: u32 = 0b001;
pub const FUNCT3_BLT: u32 = 0b100;
pub const FUNCT3_BGE: u32 = 0b101;
pub const FUNCT3_BLTU: u32 = 0b110;
pub const FUNCT3_BGEU: u32 = 0b111;

// Load and store widths
pub const FUNCT3_B: u32 = 0b000;
pub const FUNCT3_H: u32 = 0b001;
pub const FUNCT3_W: u32 = 0b010;
pub const FUNCT3_BU: u32 = 0b100;
pub const FUNCT3_HU: u32 = 0b101;
pub const FUNCT3_WU: u32 = 0b110;
pub const FUNCT3_D: u32 = 0b011;

// Register-immediate opcodes
pub const FUNCT3_ADDI: u32 = 0b000;
pub const FUNCT3_SLTI: u32 = 0b010;
pub const FUNCT3_SLTIU: u32 = 0b011;
pub const FUNCT3_XORI: u32 = 0b100;
pub const FUNCT3_ORI: u32 = 0b110;
pub const FUNCT3_ANDI: u32 = 0b111;
pub const FUNCT3_SLLI: u32 = 0b001;
pub const FUNCT3_SRLI: u32 = 0b101;
pub const FUNCT3_SRAI: u32 = 0b101;

// Register-register opcodes
pub const FUNCT3_ADD: u32 = 0b000;
pub const FUNCT3_SUB: u32 = 0b000;
pub const FUNCT3_SLL: u32 = 0b001;
pub const FUNCT3_SLT: u32 = 0b010;
pub const FUNCT3_SLTU: u32 = 0b011;
pub const FUNCT3_XOR: u32 = 0b100;
pub const FUNCT3_SRL: u32 = 0b101;
pub const FUNCT3_SRA: u32 = 0b101;
pub const FUNCT3_OR: u32 = 0b110;
pub const FUNCT3_AND: u32 = 0b111;
pub const FUNCT3_ANDW: u32 = 0b000;
pub const FUNCT3_SUBW: u32 = 0b000;
pub const FUNCT3_SLLW: u32 = 0b001;
pub const FUNCT3_SRLW: u32 = 0b101;
pub const FUNCT3_SRAW: u32 = 0b101;
pub const FUNCT3_MUL: u32 = 0b000;
pub const FUNCT3_MULH: u32 = 0b001;
pub const FUNCT3_MULHSU: u32 = 0b010;
pub const FUNCT3_MULHU: u32 = 0b011;
pub const FUNCT3_DIV: u32 = 0b100;
pub const FUNCT3_DIVU: u32 = 0b101;
pub const FUNCT3_REM: u32 = 0b110;
pub const FUNCT3_REMU: u32 = 0b111;

// Register-register opcodes
pub const FUNCT7_SUB: u32 = 0b0100000;
pub const FUNCT7_SRA: u32 = 0b0100000;
pub const FUNCT7_MULDIV: u32 = 0b0000001;
