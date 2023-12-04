/*
This linker script is based on the information provided
here: "https://interrupt.memfault.com/blog/how-to-write-
linker-scripts-for-firmware".

The memory map for the RAM region is as follows:

.data (initialised data): [0x20000000, __bss_start)
.bss section (uninitialised data): [__bss_start, _end)
heap: [_end, <undefined>)
stack: [<undefined>, ram_top - 16) 

*/

MEMORY
{
  eeprom   (rx)  : ORIGIN = 0x00000000, LENGTH = 0x00400000
  ram      (rwx) : ORIGIN = 0x20000000, LENGTH = 0x00400000
}

STACK_SIZE = 0x2000;

SECTIONS
{
    /* top of stack is at the end of RAM */
    __stacktop = ORIGIN(ram) + LENGTH(ram) - 16;

    .text :
    {
        KEEP(*(.vectors .vectors.*))
        *(.text*)
        *(.rodata*)
	/*
	Note that _etext marks the end of instructions and read-only
	data, and so marks the start of the initialised data section
	(which is stored in the eeprom section below). To initialise
	data, copy the block starting at _etext to the region
	[_sdata, _edata) (taking the size from that block). Aligned
	to ensure that _etext really is the start of the initialised
	data (and padding is not added by the linker)
	*/
	. = ALIGN(16);
	_etext = .;
    } > eeprom


    .data :
    {
	_sdata = .;
	/*
	See https://gnu-mcu-eclipse.github.io/arch/riscv/programmer/ 
	for the meaning of __global_pointer$.
	*/
	PROVIDE( __global_pointer$ = . + (4K / 2) );
	*(.sdata .sdata.*)

	*(.data*);
	_edata = .;
    } > ram AT > eeprom

    /* .bss section which is used for uninitialized data */
    .bss (NOLOAD):
    {
	__bss_start = .;
        *(.bss*)
	_end = .;
        *(COMMON)
    } > ram

    /* stack section */
    /*
    . = ORIGIN(RAM) + 
    .stack (NOLOAD):
    {
        . = ALIGN(16);
        . = . + STACK_SIZE;
        . = ALIGN(16);
    } > ram

    _estack = .;
    */
}
