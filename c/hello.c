
static volatile char *dev = (char*)0x3f8;

int main() {
    *dev = 'h';
    while (1)
	;
}


