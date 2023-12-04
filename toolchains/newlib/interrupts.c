#include "interrupts.h"

void set_timeout(uint64_t period) {
    volatile uint64_t *mtime = (uint64_t*)MTIME_BASE;
    volatile uint64_t *mtimecmp = (uint64_t*)MTIMECMP_BASE;
    *mtimecmp = *mtime + period;
}
