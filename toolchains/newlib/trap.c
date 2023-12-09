#include <stdio.h>
#include "interrupts.h"
#include "csr.h"

#define MCAUSE_INSTRUCTION_ADDRESS_MISALIGNED 0
#define MCAUSE_INSTRUCTION_ACCESS_FAULT 1
#define MCAUSE_ILLEGAL_INSTRUCTION 2
#define MCAUSE_BREAKPOINT 3
#define MCAUSE_LOAD_ADDRESS_MISALIGNED 4
#define MCAUSE_LOAD_ACCESS_FAULT 5
#define MCAUSE_STORE_ADDRESS_MISALIGNED 6
#define MCAUSE_STORE_ACCESS_FAULT 7
#define MCAUSE_MMODE_ECALL 11

void _nmi_handler() {
    printf("nmi");
    while (1)
	;
}

inline __attribute__((always_inline)) int read_csr(int csr_num) {
    int result;
    asm("csrr %0, %1"
	: "=r"(result)
	: "I"(csr_num));
    return result;
}

inline __attribute__((always_inline)) void write_csr(int csr_num, int value) {
    asm("csrw %0, %1"
	: /* no outputs */
	: "I"(csr_num), "r"(value));
}


void _exception_handler() {

    // You cannot use stdout inside this function because the C
    // environment is not necessarily set up yet.
    
    int mcause = read_csr(CSR_MCAUSE);

    // Assume that mcause does not have the interrupt bit set
    switch(mcause) {
    case MCAUSE_INSTRUCTION_ADDRESS_MISALIGNED:
	break;
    case MCAUSE_INSTRUCTION_ACCESS_FAULT:
	break;
    case MCAUSE_ILLEGAL_INSTRUCTION:
	break;
    case MCAUSE_BREAKPOINT:
	break;
    case MCAUSE_LOAD_ADDRESS_MISALIGNED:
	break;
    case MCAUSE_LOAD_ACCESS_FAULT:
	break;
    case MCAUSE_STORE_ADDRESS_MISALIGNED:
	break;
    case MCAUSE_STORE_ACCESS_FAULT:
	break;
    case MCAUSE_MMODE_ECALL:
	// The ecall is an nop in this platform
	write_csr(CSR_MEPC, read_csr(CSR_MEPC) + 4);
        break;
    default:
	// Unknown mcause
	while(1)
	    ;
    }
    asm("mret");
}

void _software_isr() {
    printf("software");
    while (1)
	;  
}
void _timer_isr() {
    printf("tick\n");
    set_timeout(2000000);
    asm("mret");
}

void _external_isr() {
    printf("external");
    while(1)
	;
}

