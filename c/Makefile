# See NOTES.md on installing this compiler
CC=riscv32-unknown-elf-gcc

hello.out: hello.c startup.o linker.ld Makefile
	$(CC) -T linker.ld -O2 -ffreestanding -nostdlib hello.c startup.o -o $@

startup.o: startup.S
	$(CC) -s $< -o startup.o
