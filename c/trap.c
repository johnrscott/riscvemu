#include "printf.h"
#include "interrupts.h" 

void _nmi_handler() {
    printf("nmi");
    while (1)
	;
}

void _exception_handler() {
    printf("exception");
    while (1)
	;
}

void _software_isr() {
    printf("software");
    while (1)
	;  
}
void _timer_isr() {
    //printf("timer");
    set_timeout(50);
    asm("mret");
}

void _external_isr() {
    printf("external");
    while(1)
	;
}

