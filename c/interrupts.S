#include "csr.h"

	.text
	
	.global global_enable_interrupts
global_enable_interrupts:
	csrrsi x0, CSR_MSTATUS, MSTATUS_MIE
	ret

	.global enable_machine_timer_interrupt
enable_machine_timer_interrupt:
	// Is there a one-instruction way to do this?
	li t0, MIE_MTIE
	csrrs x0, CSR_MIE, t0
	ret
	
