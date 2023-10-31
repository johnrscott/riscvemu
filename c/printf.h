/**
 * \brief C input/output standard library
 *
 *
 */

/**
 * \brief Slimmed down version of putchar - '\n' is the only
 * special character it supports 
 */
int putchar(int c);

/**
 * \brief Print a string
 */
int puts(const char* str);

int printf(const char* format, ...);
