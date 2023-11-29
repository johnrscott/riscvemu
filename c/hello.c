#include "printf.h"
#include "interrupts.h"

int main() {

    set_timeout(50);
    
    // Enable timer interrupt
    enable_machine_timer_interrupt();
    global_enable_interrupts();

    printf("Enabled timer");
    while (1)
	;
}
