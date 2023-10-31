#include <string.h>

void putchar(char ch) {
    // This emits a warning currently, might be a
    // compiler bug:
    // https://stackoverflow.com/questions/71383351/
    // how-to-avoid-wrong-array-bounds-warning-on-a-pointer-in-g12
    static volatile char *dev = (char*)0x3f8;
    *dev = ch;
}

/**
 * \brief Print a string
 */
int puts(const char * str)
{
    for (size_t i = 0; i < 10; i++) {
	putchar(str[i]);
    }

    return 0;
}

int triangle_number(int n) {
    if (n == 0) {
	return 0;
    } else {
	return n + triangle_number(n-1);
    }
}

int divide(int a, int b) {
    return a / b;
}

int main() {
    //int m = divide(6, 2);
    putchar('a');
    puts("Hello");
    while (1)
	;
}
