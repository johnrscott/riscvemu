//#include <string.h>
#include <stdbool.h>
#include <stdarg.h>
#include <stddef.h>

size_t strlen(const char* str) 
{
    size_t len = 0;
    while (str[len] != '\0') {   // \0 is the NULL char
	len++;
    }
    return len;
}

void putchar(char ch) {
    // This emits a warning currently, might be a
    // compiler bug:
    // https://stackoverflow.com/questions/71383351/
    // how-to-avoid-wrong-array-bounds-warning-on-a-pointer-in-g12
    static volatile int *dev = (int*)0x00010018;
    *dev = (int)ch;
}

/**
 * \brief Print a string
 */
int puts(const char * str)
{
    for (size_t i = 0; i < strlen(str); i++) {
        putchar(str[i]);
    }

    return 0;
}

void reverse(char* str)
{
    char temp;
    size_t len = strlen(str);
    for (size_t i = 0; i < len/2; i++) {
	size_t j = len - 1 - i;
	temp = str[i];
	str[i] = str[j];
	str[j] = temp;
    }    
}

char * itoa(int value, char * str)
{
    int i = 0;
    bool isNegative = false;
    unsigned int num = 0;
    int base = 10;

    // Handle 0 explicitly, otherwise an empty string will be printed.
    if (value == 0) {
	str[i++] = '0';
	str[i] = '\0';
	return str;
    }

    // Numbers are usually considered unsigned.
    // Negative numbers are handled only with base 10 in standard itoa.
    if (value < 0) {
	isNegative = true;
	num = -value;
    } else {
	num = (unsigned int) value;
    }
	
    // Process individual digits in reverse order.
    while (num != 0) {
	int remainder = num % base;
	if (remainder > 9) {
	    str[i++] = (remainder - 10) + 'a';
	} else {
	    str[i++] = remainder + '0';
	}
      	num = num/base;
    }

    // If number is negative, append '-'.
    if (isNegative == true) {
	str[i++] = '-';
    }

    // Append string terminator and reverse to correct order.
    str[i] = '\0';
    reverse(str);
    
    return str;
}

int printf(const char * format, ...)
{
    // Define a variable to manipulate extra arguments
    va_list args;
    // Initializes args variable to the last fixed argument
    va_start(args, format);

    char str[32];
    
    for (const char *p = format; *p != '\0'; p++) {
	// Print character if it is not a %
        if (*p != '%') {
	    putchar(*p);
	}
	else {
	    // Increment to character after %
	    p++;

	    switch(*p) {
	    case 'd': {   // Decimal number
		int i = va_arg(args, int);
		puts(itoa(i, str));
		break;
	    }
	    case 's': {   // String
		char *s = va_arg(args, char *);
		puts(s);
		break;
	    }
	    case '%': {
		putchar('%');
		break;
	    }
	    default: {   // Otherwise assume it is a string
		char *s = va_arg(args, char *);
		puts(s);
		break;
	    }	
	    }
	}
    }

    // Close argument list and clean up 
    va_end(args);
    
    /// \todo Return the number of characters written
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
    const char * hello = "Hello world!";
    printf("%s, %d", hello, 10);
    while (1)
	;
}
