/* startup.S contains the code that runs first */
STARTUP(startup.o)

/* Memory is 4k in total */
MEMORY {
	instr : ORIGIN = 0x0, LENGTH = 1016,
	io : ORIGIN = 0x3f8, LENGTH = 8,
	data : ORIGIN = 0x400, LENGTH = 2048,
	stack : ORIGIN = 0xc00, LENGTH = 1024
}

SECTIONS {
	 .text : {
	       	 *(.text.startup)
		 *(.text)
		 }
    	 .data : { *(.data) } > data
    	 .bss : { *(.bss) } > data
}
