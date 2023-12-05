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

/*
Note that you cannot assume the .data section will be placed at the
end of .text section directly after .rodata*. For example, the section
.srodata.cst8 appeared before .data, meaning that starting copying
initialised data from the end of the text section is an
error. Instead, LOADADDR is used to identify the LMA of the .data
section, which provides a reliable way to identify the load address.
*/
_data_load_address = LOADADDR(.data);

SECTIONS
{
    /* top of stack is at the end of RAM */
    __stacktop = ORIGIN(ram) + LENGTH(ram) - 16;

    .text :
    {
        KEEP(*(.vectors .vectors.*))
        *(.text*)
        *(.rodata*)
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
	/* 
	This feels like a hack -- needed to add in .sbss here because
	the linker was outputting it as a separate section, which was
	messing up _end. Really want a robust way to identify the end
	of _all_ uninitialised data sections.
	*/
        *(.bss* .sbss*)
        *(COMMON)
	_end = .;
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
