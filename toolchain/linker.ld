/* startup.S contains the code that runs first */
STARTUP(startup.o)

MEMORY {
	instr : ORIGIN = 0x0, LENGTH = 4M,
	io : ORIGIN = 0x10000, LENGTH = 0x80,
	data : ORIGIN = 0x20000, LENGTH = 4M
}


SECTIONS {
	 .text : {
	       	 *(.text.startup)
		 *(.text)
		 } > instr
    	 .data : { *(.data) } > data
    	 .bss : { *(.bss) } > data
}
