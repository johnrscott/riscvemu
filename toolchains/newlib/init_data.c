#include <stdint.h>

extern uint32_t _data_load_address;
extern uint32_t _sdata;
extern uint32_t _edata;

void _initialise_data() {
    uint32_t *init_values_ptr = &_data_load_address;
    uint32_t *data_ptr = &_sdata;
    if (init_values_ptr != data_ptr) {
	for (; data_ptr < &_edata;) {
	    *data_ptr++ = *init_values_ptr++;
	}
    }
}
