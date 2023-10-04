# See NOTES.md on installing both these compilers
CC=riscv64-unknown-linux-gnu-gcc
KK=kcc

all: add_memory.out

check: add_memory_kcc.out

%.out: %.c Makefile
	$(CC) -O1 -c $< -o $@

%_kcc.out: %.c Makefile
	$(KK) -O1 -c $< -o $@

