// Control/status register (CSR) addresses
#define CSR_MSTATUS 0x300
#define CSR_MIE 0x304

// CSR fields
#define MSTATUS_MIE (1 << 3)
#define MIE_MTIE (1 << 7)
