# See NOTES.md on installing this compiler
CC=riscv64-unknown-linux-gnu-gcc

CFLAGS=-O2 -Wall -Wextra -pedantic

all: add_memory.out

%.out: %.c Makefile
	$(CC) $(CFLAGS) -c $< -o $@
