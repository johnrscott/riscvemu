/* startup.S contains the code that runs first */
STARTUP(startup.o)

SECTIONS {
	 .text : { *(.text) }

	 . = 0x20000000;
	 PROVIDE(__bss_start = .);
	 .bss : { *(.bss) }

	 .data : {
	 	 /*
	 	 See https://gnu-mcu-eclipse.github.io/arch/riscv/programmer/ 
	 	 for the meaning of __global_pointer$.
	 	 */
	 	 PROVIDE( __global_pointer$ = . + (4K / 2) );
	 	 *(.sdata .sdata.*)

		 *(.data)
	 }

	 /*
	 See https://sourceware.org/newlib/ for the
	 meaning of _end -- defined in sbrk_ stub,
	 marks the end of the heap.
	 */
	 . = . + 1M;
	 PROVIDE(_end = .);	
}
