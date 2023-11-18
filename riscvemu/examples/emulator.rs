use riscvemu::{decode::Decoder, rv32i::make_rv32i, rv32m::make_rv32m, hart::Hart, elf_utils::load_elf};
    use std::time::{Instant, Duration};

fn main() {
    
    // Make an RV32IM hart
    let mut decoder = Decoder::default();
    make_rv32i(&mut decoder).expect("adding instructions should work");
    make_rv32m(&mut decoder).expect("adding instructions should work");
    let mut hart = Hart::new(decoder);

    // Open an executable file
    let elf_name = "c/hello.out".to_string();
    load_elf(&mut hart, &elf_name);

    let total = 1_000_000;
    println!("Executing {total} instructions");

    let now = Instant::now();
    for _ in 0..total {
	hart.step().unwrap();
    }
    let elapsed = now.elapsed();
    
    let hart_stdout = hart.memory.flush_stdout();
    println!("Hart stdout: {hart_stdout}");
    
    let time_per_instruction = elapsed / total;
    println!("Time per instruction: {time_per_instruction:.2?}");

}
