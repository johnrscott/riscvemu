# See NOTES.md on installing this compiler
CC=riscv64-unknown-linux-gnu-gcc

all: add_memory.out

%.out: %.c Makefile
	$(CC) -O3 -c $< -o $@
