// Control/status register (CSR) addresses
#define CSR_MSTATUS 0x300
#define CSR_MIE 0x304
#define CSR_MEPC 0x341
#define CSR_MCAUSE 0x342

// CSR fields
#define MSTATUS_MIE (1 << 3)
#define MIE_MTIE (1 << 7)
