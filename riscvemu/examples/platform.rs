use riscvemu::{elf_utils::load_elf, hart::platform::Platform};

fn main() {
    let mut platform = Platform::new();

    // Open an executable file
    let elf_name = "../c/hello.out".to_string();
    load_elf(&mut platform, &elf_name);

    println!("{platform:?}");
}
