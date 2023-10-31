# See NOTES.md on installing this compiler
CC=riscv32-unknown-elf-gcc

# Note startup.o is added by the linker.ld script
hello.out: hello.c startup.o linker.ld Makefile
	$(CC) -T linker.ld -Os -Wall -Wextra -ffreestanding -nostartfiles hello.c printf.c -o $@

startup.o: startup.S
	$(CC) -s $< -o startup.o
