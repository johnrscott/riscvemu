#include "printf.h"

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
    int m = divide(6, 2);
    //putchar(0x30 + m);
    printf("Hello");
    while (1)
	;
}
