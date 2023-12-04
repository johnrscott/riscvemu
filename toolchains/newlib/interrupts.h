#include <stdint.h>

#define MTIME_BASE 0x10000
#define MTIMECMP_BASE 0x10008

void global_enable_interrupts();
void enable_machine_timer_interrupt();

void set_timeout(uint64_t period);
