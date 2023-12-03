/**
 * \file newlib.c
 * \brief Implementation of newlib stubs for the platform
 *
 * See https://sourceware.org/newlib/ for documentation on
 * implementing the newlib functions.
 * 
 */

static void outbyte(char c) {
    static volatile int *dev = (int*)0x10000018;
    *dev = (int)c;
}

int _write(int file, char *buf, int nbytes) {

    /* Output character at at time */
    for (int i = 0; i < nbytes; i++) {
	outbyte(buf[i]);
    }

    return nbytes;
}
