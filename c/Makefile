# See NOTES.md on installing this compiler
CC=riscv64-unknown-elf-gcc
CFLAGS=-march=rv32i -mabi=ilp32
LDFLAGS=-march=rv32i -mabi=ilp32

# Note startup.o is added by the linker.ld script.
# -ffreestanding enables the C freestanding (i.e. not
# hosted in an OS) environment, which may still use
# the std lib. To disable the std lib completely, use
# -nostdlib. In that case, you may want to link -lgcc
# to keep compiler mulation of missing instructions.
# If not using -nostdlib, use -nostartfiles to turn
# off crt0 etc. See the manual page for gcc for full
# information.
hello.out: hello.c startup.o linker.ld Makefile
	$(CC) $(CFLAGS) -T linker.ld -flto -Os -Wall -Wextra -ffreestanding -nostartfiles hello.c -o $@

startup.o: startup.S
	$(CC) $(CFLAGS) -c $< -o startup.o
