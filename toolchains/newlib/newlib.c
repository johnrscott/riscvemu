/**
 * \file newlib.c
 * \brief Implementation of newlib stubs for the platform
 *
 * See https://sourceware.org/newlib/ for documentation on
 * implementing the newlib functions.
 *
 * See also the reference implementations in the libgloss directory in
 * the riscv-gnu-toolchain folder: newlib/libgloss/riscv/sys_*.
 *
 * These implementations are based on the versions described here
 * https://interrupt.memfault.com/blog/boostrapping-libc-with-newlib
 */

#include <sys/stat.h>
#include <stddef.h>

/// This is the beginning of the heap (which begins directly after the
/// .bss section, hence the use of the _end symbol).
extern int _end;

void *_sbrk(int incr) {
    static unsigned char *heap = NULL;
    unsigned char *prev_heap;
    
    if (heap == NULL) {
	heap = (unsigned char *)&_end;
    }
    prev_heap = heap;
    
    heap += incr;
    
    return prev_heap;
}

int _close(__attribute__((unused)) int fd) {
    return -1;
}

int _fstat(__attribute__((unused)) int file, struct stat *st) {
    st->st_mode = S_IFCHR;
    return 0;
}

int _isatty(__attribute__((unused)) int file) {
    return 1;
}

int _lseek(__attribute__((unused)) int file, __attribute__((unused)) int offset,
           __attribute__((unused)) int whence)
{
    return 0;
}

/*
void _exit(int status) {
    __asm("BKPT #0");
}
*/

void _kill(__attribute__((unused)) int pid, __attribute__((unused)) int sig) {
    return;
}

int _getpid(void) {
    return -1;
}

static void outbyte(char c) {
    static volatile int *dev = (int*)0x10000018;
    *dev = (int)c;
}

int _write(__attribute__((unused)) int file, char *buf, int nbytes) {

    /* Output character at at time */
    for (int i = 0; i < nbytes; i++) {
	outbyte(buf[i]);
    }

    return nbytes;
}
