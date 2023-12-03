ISA=rv32imzicsr

# See NOTES.md on installing this compiler
CC=riscv64-unknown-elf-gcc
CFLAGS=-march=$(ISA) -mabi=ilp32
LDFLAGS=-march=$(ISA) -mabi=ilp32

# Note startup.o is added by the linker.ld script.  -ffreestanding
# enables the C freestanding (i.e. not hosted in an OS) environment,
# which may still use the std lib. To disable the std lib completely,
# use -nostdlib. In that case, you may want to link -lgcc to keep
# compiler emulation of missing instructions.  If not using -nostdlib,
# use -nostartfiles to turn off crt0 etc. See the manual page for gcc
# for full information. trap.c is near the start of the list so that
# the handlers are placed at low addresses, so that the jumps from the
# trap vector table are in range.
#
# See https://gnu-mcu-eclipse.github.io/arch/riscv/programmer/ for a
# description of the global pointer, which enables a memory
# optimisation. Pass -Wl,--no-relax to disable it (you still need to
# define __global_pointer$, but it can be zero)
main.out: main.c trap.c startup.o newlib.c linker.ld Makefile
	$(CC) $(CFLAGS) -T linker.ld -flto -Os -Wall -Wextra \
		-ffreestanding main.c newlib.c -o $@

startup.o: startup.S Makefile
	$(CC) $(CFLAGS) -c $< -o startup.o

interrupts.o: interrupts.S Makefile
	$(CC) $(CFLAGS) -c $< -o interrupts.o

.PHONY: clean

clean:
	rm -rf *.o main.out
