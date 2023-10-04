use crate::cpu::Cpu;

pub fn write_constant_vector(
    cpu: &mut Cpu,
    value: u64,
    value_byte_width: usize,
    start_addr: usize,
    end_addr: usize,
) {
    for addr in (start_addr..end_addr).step_by(value_byte_width) {
        cpu.write_data(addr, value, value_byte_width);
    }
}
