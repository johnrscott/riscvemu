	.text
	.global _vector_table
_vector_table:
	j _setup_stack // reset vector	

	.global _setup_stack
_setup_stack:
	// Full memory is 4 MiB long (first invalid address is
	// 0x2040_0000), stack pointer sp must be 16-byte aligned
	// so use last valid aligned address as start of stack
	li sp, 0x203FFFF0
	tail _start
