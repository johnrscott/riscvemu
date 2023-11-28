ISA=rv32im

# See NOTES.md on installing this compiler
CC=riscv64-unknown-elf-gcc
CFLAGS=-march=$(ISA) -mabi=ilp32
LDFLAGS=-march=$(ISA) -mabi=ilp32

# Note startup.o is added by the linker.ld script.  -ffreestanding
# enables the C freestanding (i.e. not hosted in an OS) environment,
# which may still use the std lib. To disable the std lib completely,
# use -nostdlib. In that case, you may want to link -lgcc to keep
# compiler mulation of missing instructions.  If not using -nostdlib,
# use -nostartfiles to turn off crt0 etc. See the manual page for gcc
# for full information. trap.c is near the start of the list so that
# the handlers are placed at low addresses, so that the jumps from the
# trap vector table are in range.
hello.out: hello.c trap.c startup.o linker.ld Makefile
	$(CC) $(CFLAGS) -T linker.ld -flto -Os -Wall -Wextra -ffreestanding -nostartfiles trap.c hello.c -o $@

startup.o: startup.S Makefile
	$(CC) $(CFLAGS) -c $< -o startup.o

.PHONY: clean

clean:
	rm -rf *.o hello.out
