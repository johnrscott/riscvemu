/* startup.S contains the code that runs first */
STARTUP(startup.o)

/* Memory is 4k in total */
MEMORY {
	instr : ORIGIN = 0x0, LENGTH = 8k,
	io : ORIGIN = 0x10000, LENGTH = 0x80,
	data : ORIGIN = 0x20000, LENGTH = 8k
}


SECTIONS {
	 .text : {
	       	 *(.text.startup)
		 *(.text)
		 } > instr
    	 .data : { *(.data) } > data
    	 .bss : { *(.bss) } > data
}
