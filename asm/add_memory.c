void add_memory() {
    volatile long *a = (long*)0;
    volatile long *b = (long*)8;
    volatile long *c = (long*)16;

    // Add up the contents of a and b and place in c
    *c = *a + *b;
}
