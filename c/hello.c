// This emits a warning currently, might be a
// compiler bug:
// https://stackoverflow.com/questions/71383351/
// how-to-avoid-wrong-array-bounds-warning-on-a-pointer-in-g12
static volatile char *dev = (char*)0x3f8;

int main() {
    *dev = '5';
    while (1)
	;
}
