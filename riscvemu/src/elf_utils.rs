use elf::abi::SHF_ALLOC;
use elf::endian::AnyEndian;
use elf::section::SectionHeader;
use elf::string_table::StringTable;
use elf::ElfBytes;

use crate::hart::memory::Wordsize;
use crate::hart::Hart;

/// Get the section header name for this section
fn section_name<'a>(header: &SectionHeader, strtab: &'a StringTable) -> &'a str {
    let index = header.sh_name;
    strtab
        .get(index.try_into().unwrap())
        .expect("name of section in string table")
}

pub fn read_text_instructions(file_path: &String) -> Vec<u32> {
    let path = std::path::PathBuf::from(file_path);
    let file_data = std::fs::read(path).expect("Could not read file.");
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");

    let (section_headers, strtab) = file
        .section_headers_with_strtab()
        .expect("section headers available");
    let section_headers = section_headers.expect("section headers are present");
    let strtab = strtab.expect("string table is present");

    for header in section_headers.iter() {
        // We are looking for executable sections to load into memory
        let flags = header.sh_flags;
        if flags & u64::from(SHF_ALLOC) != 0 {
            let section_name = section_name(&header, &strtab);
            println!("Found section to load for execution: {section_name}");
            println!("{:x?}", header);
        }
        //SHF_ALLOC & SHF_EXECINSTR;
    }

    let text_shdr: SectionHeader = file
        .section_header_by_name(".text")
        .expect("section .text should be parseable")
        .expect("file should have a .text section");

    // Byte stream of text section
    let data_pair = file
        .section_data(&text_shdr)
        .expect("valid section data in .text");
    if data_pair.1.is_some() {
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

/// Returns offset, assumed .text section currently. Returns the symbol start address
/// and the length of the symbol (the number of bytes of .text it occupies)
pub fn find_function_symbol(file_path: &String, symbol_name: &String) -> Option<(usize, usize)> {
    let path = std::path::PathBuf::from(file_path);
    let file_data = std::fs::read(path).expect("Could not read file.");
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");

    let (symtab, strtab) = file
        .symbol_table()
        .expect("symbol table to parse")
        .expect("symbol table to be present");

    for entry in symtab.iter() {
        if entry.st_symtype() == elf::abi::STT_FUNC {
            let name_strtab_index = entry.st_name;
            let name = strtab
                .get(name_strtab_index.try_into().unwrap())
                .expect("Valid string table entry at index");
            if name == symbol_name {
                return Some((
                    entry.st_value.try_into().unwrap(),
                    entry.st_size.try_into().unwrap(),
                ));
            }
        }
    }
    None
}

fn section_data<'a>(
    header: &SectionHeader,
    file: &'a ElfBytes<'_, AnyEndian>,
) -> &'a [u8] {
    let data_pair = file
        .section_data(header)
        .expect("valid section data corresponding to the section header");
    if data_pair.1.is_some() {
        panic!("found unexpected compression in .text section")
    }
    data_pair.0
}

/// Read an ELF file from disk and load the alloc section (the ones
/// meant to be present during program execution) into memory. Prints
/// what it is doing.
pub fn load_elf(hart: &mut Hart, elf_file_path: &String) {
    let path = std::path::PathBuf::from(elf_file_path);
    let file_data = std::fs::read(path).expect("Could not read file.");
    let slice = file_data.as_slice();
    let file = ElfBytes::<AnyEndian>::minimal_parse(slice).expect("Open test1");

    let (section_headers, strtab) = file
        .section_headers_with_strtab()
        .expect("section headers available");
    let section_headers = section_headers.expect("section headers are present");
    let strtab = strtab.expect("string table is present");

    for header in section_headers.iter() {
        // We are looking for executable sections to load into memory
        let flags = header.sh_flags;
        if flags & u64::from(SHF_ALLOC) != 0 {
            let section_name = section_name(&header, &strtab);
            println!("Found section to load for execution: {section_name}");
            println!("{:x?}", header);
            let data = section_data(&header, &file);
            let section_load_address = header.sh_addr;

            for (offset, byte) in data.iter().enumerate() {
                let addr = section_load_address + u64::try_from(offset).unwrap();
                hart.memory
                    .write(addr, (*byte).into(), Wordsize::Byte)
                    .unwrap();
            }
        }
    }
}
