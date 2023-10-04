void add_memory() {
    // Make sure that you don't initialise a pointer to 0 and
    // dereference it, which would be undefined behaviour.
    // "https://stackoverflow.com/questions/26309300/c-code-with-
    // undefined-results-compiler-generates-invalid-code-with-o3"
    volatile long *a = (long*)8;
    volatile long *b = (long*)16;
    volatile long *c = (long*)24;

    // Add up the contents of a and b and place in c
    *c = *a + *b;
}
