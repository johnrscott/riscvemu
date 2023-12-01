	.text
	.section .text.startup

	.global _vector_table
_vector_table:
	j _setup_stack // reset vector
	j _nmi_handler // NMI vector
	j _exception_handler // trap vector table base address
	nop
	nop
	j _software_isr // machine software interrupt vector
	nop
	nop
	nop
	j _timer_isr // machine timer interrupt vector
	nop
	nop
	nop
	j _external_isr // machine external interrupt vector
	

	.global _setup_stack
_setup_stack:
	// Full memory is 8 kiB long (first invalid address is
	// 0x0002_2000), stack pointer sp must be 16-byte aligned
	// so use last valid aligned address as start of stack
	li sp, 0x00021ff0
	tail main
