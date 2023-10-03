use elf::endian::AnyEndian;
use elf::section::SectionHeader;
use elf::ElfBytes;

pub fn read_text_instructions(file_path: &String) -> Vec<u32> {
    let path = std::path::PathBuf::from(file_path);
    let file_data = std::fs::read(path).expect("Could not read file.");
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");

    let text_shdr: SectionHeader = file
        .section_header_by_name(".text")
        .expect("section .text should be parseable")
        .expect("file should have a .text section");

    // Byte stream of text section
    let data_pair = file
        .section_data(&text_shdr)
        .expect("valid section data in .text");
    if data_pair.1 != None {
        panic!("Unexpected compression in .text section")
    }
    let data = data_pair.0;

    // Data is a little-endian byte stream. Reinterpret it
    // as a stream of 32-bit words
    let mut instructions = Vec::new();
    for n in (0..data.len()).step_by(4) {
        instructions.push(u32::from_le_bytes(data[n..(n + 4)].try_into().unwrap()));
    }

    instructions
}
