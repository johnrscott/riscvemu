/**
 * \brief C input/output standard library implementation
 *
 *
 */

#include <stdlib.h>
#include <stdbool.h>
#include <string.h>
#include <stdarg.h>

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

char * itoa_internal(int value, char * str, int base)
{
    int i = 0;
    bool isNegative = false;
    unsigned int num = 0;
    
    // Check base is between 2 and 36
    if (base < 2 || base > 36) {
	str[i] = '\0';
	return str;
    }

    // Handle 0 explicitly, otherwise an empty string will be printed.
    if (value == 0) {
	str[i++] = '0';
	str[i] = '\0';
	return str;
    }

    // Numbers are usually considered unsigned.
    // Negative numbers are handled only with base 10 in standard itoa.
    if (value < 0 && base == 10) {
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
    for (size_t i = 0; i < strlen(str); i++) {
	putchar(str[i]);
    }

    return 0;
}

int printf(const char * format, ...)
{
    // Define a variable to manipulate extra arguments
    va_list args;
    // Initializes args variable to the last fixed argument
    va_start(args, format);

    char str[100];
    bool precision = false;
    unsigned int prec_len = 0;
    
    for (const char *p = format; *p != '\0'; p++) {
	// Print character if it is not a %
        if (*p != '%') {
	    putchar(*p);
	}
	else {
	    // Increment to character after %
	    p++;

	    // Check if precision has been specified
	    // Only supported for string at the moment
	    /// \todo Extend precision specification to numbers
	    if (*p == '.') {
		// Precision specified
		precision = true;
		p++;
		// Precision is specified in an argument
		if (*p == '*') {
		    prec_len = va_arg(args, unsigned int);
		    p++;
		}
	    }
	    
	    switch(*p) {
	    case 'd': {   // Decimal number
		int i = va_arg(args, int);
		puts(itoa_internal(i, str, 10));
		break;
	    }
	    case 'x': {   // Hexadecimal number
		unsigned int i = va_arg(args, unsigned int);
		if (precision == false) {
		    puts(itoa_internal(i, str, 16));
		}
		else {
		    char * num_str = itoa_internal(i, str, 16);
		    if (strlen(num_str) >= prec_len) {
			puts(num_str);
		    }
		    else {
			for (size_t i = 0; i < prec_len - strlen(num_str); i++) {
			    putchar('0');
			}
			puts(num_str);
		    }
		}
		break;
	    }
	    case 'o': {   // Octal number
		unsigned int i = va_arg(args, unsigned int);
		puts(itoa_internal(i, str, 8));
		break;
	    }
	    case 'b': {   // Binary number
		unsigned int i = va_arg(args, unsigned int);
		puts(itoa_internal(i, str, 2));
		break;
	    }
	    case 's': {   // String
		char *s = va_arg(args, char *);
		if (precision == false) {
		    puts(s);
		}
		else {
		    for (unsigned int i = 0; i < prec_len; i++) {
			putchar(s[i]);
		    }
		    // Reset precision flag
		    precision = false;
		}
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
