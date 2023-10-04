use elf::endian::AnyEndian;
use elf::section::SectionHeader;
use elf::ElfBytes;

use crate::cpu::Cpu;

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

pub fn read_all_symbols(file_path: &String) {
    let path = std::path::PathBuf::from(file_path);
    let file_data = std::fs::read(path).expect("Could not read file.");
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");

    let (symtab, strtab) = file
        .symbol_table()
        .expect("symbol table to parse")
        .expect("symbol table to be present");

    println!("{:?}", symtab);
    
    let common = file.find_common_data().expect("shdrs should parse");
    // let dynsyms = common.dynsyms.unwrap();
    // let strtab = common.dynsyms_strs.unwrap();
    let hash_table = common.gnu_hash.unwrap();
    // Use the hash table to find a given symbol in it.
    let name = b"memset";
    let (sym_idx, sym) = hash_table.find(name, &symtab, &strtab)
	.expect("hash table and symbols should parse").unwrap();
}

pub fn load_text_section(cpu: &mut Cpu, elf_file_path: &String) {
    let instructions = read_text_instructions(elf_file_path);

    let mut addr = 0;
    for instr in instructions {
        cpu.write_instruction(addr, instr);
        addr += 4;
    }
}
