	.section .vectors, "x"
_vectors:
	j _reset_handler // reset vector
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

	
	.text
	// The newlib _start function is defined in crt0.S
	// (riscv-gnu-toolchain/newlib/libgloss/riscv/crt0.S).
	// It sets the bss segment to zero (defined as the region
	// between the ___bss_start and _end symbols, which must
	// be defined in the linker script).
	//
	// The newlib crt0.S does not copy the initialised data (see
	// the discussion here: "https://newlib.sourceware.narkive.com/9geROyjD/
	// initialize-data-section-in-arm-crt0-s"). As a result, it
	// is necessary to copy this section manually. After setting
	// up the stack, it is possible to call functions, so the
	// data initialisation routine is written is C.
_reset_handler:
	la sp, __stacktop

	// Before calling functions compiled by gcc, make sure
	// the global pointer is initialised (as it may be used).
	// These lines are taken from here: "https://www.five-embeddev.com//
	// quickref/global_pointer.html". Maybe it is better to just
	// write the data section initialisation in assembly to avoid
	// having to do this.
	.option push
	.option norelax
	la gp, __global_pointer$
	.option pop
	
	// Copy the data section from eeprom into ram
	call _initialise_data

	// Jump to newlib initialisation
	tail _start
