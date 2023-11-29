#include "printf.h"
#include "interrupts.h"

int main() {

    set_timeout(2000000);
    
    // Enable timer interrupt
    enable_machine_timer_interrupt();
    global_enable_interrupts();

    printf("Enabled timer!\n");
    while (1)
	;
}
