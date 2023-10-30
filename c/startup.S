	.text
	.section .text.startup

	// _start symbol will be placed at memory address 0,
	// which is where execution will begin
	.global _start
_start:
	// Full memory is 4k long (first invalid address is
	// 0x1000), stack pointer sp must be 16-byte aligned
	// so use last valid aligned address as start of stack
	li sp, 0xff0
	tail main
