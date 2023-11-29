#include "printf.h"
#include "interrupts.h"

#define MTIME_BASE 0x10000
#define MTIMECMP_BASE 0x10008

int main() {

    volatile int *mtime = (int*)MTIME_BASE;
    volatile int *mtimecmp = (int*)MTIMECMP_BASE;

    // Set timeout
    *mtimecmp = *mtime + 50;

    // Enable timer interrupt
    enable_machine_timer_interrupt();
    global_enable_interrupts();

    printf("Enabled timer");
    while (1)
	;
}
