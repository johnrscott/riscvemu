/* startup.S contains the code that runs first */
STARTUP(startup.o)

SECTIONS {
	 .text : { *(.text) }

	 . = 0x20000000;
	 PROVIDE(__bss_start = .);
	 .bss : { *(.bss) }
	 .data : { *(.data) }

	 /*
	 See https://sourceware.org/newlib/ for the
	 meaning of _end -- defined in sbrk_ stub,
	 marks the end of the heap.
	 */
	 . = . + 1M;
	 PROVIDE(_end = .);	
	 PROVIDE(__global_pointer$ = 0);
}
