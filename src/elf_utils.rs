use std::io;

use elf::abi::{
    SHF_ALLOC, SHF_WRITE, SHN_COMMON, SHN_UNDEF, STB_GLOBAL, STB_HIPROC,
    STB_LOCAL, STB_LOPROC, STB_WEAK, STT_FILE, STT_FUNC, STT_HIPROC,
    STT_LOPROC, STT_NOTYPE, STT_OBJECT, STT_SECTION,
};
use elf::endian::AnyEndian;
use elf::section::{SectionHeader, SectionHeaderTable};
use elf::segment::{ProgramHeader, SegmentTable};
use elf::string_table::StringTable;
use elf::symbol::{Symbol, SymbolTable};
use elf::{ElfBytes, ParseError};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ElfError {
    #[error("Attempted to write byte to non-writable memory address 0x{0:x}")]
    NonWritable(u32),
    #[error("Failed to open file: {0}")]
    CouldNotOpenFile(String),
    #[error("Failed to parse ELF format: {0}")]
    ParseError(String),
    #[error("Symbol table is missing")]
    MissingSymbolTable,
    #[error("Could not find .text section")]
    MissingTextSection,
    #[error("Could not find section header table")]
    MissingSectionHeaderTable,
    #[error("Could not find string table")]
    MissingStringTable,
    #[error("st_bind value has invalid value {0}")]
    InvalidSymbolInfoBind(u8),
    #[error("st_type value has invalid value {0}")]
    InvalidSymbolInfoType(u8),
    #[error("missing program segment table")]
    MissingSegmentTable,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SymbolBind {
    Local,
    Global,
    Weak,
    Loproc,
    Hiproc,
}

impl SymbolBind {
    fn from_st_bind(st_bind: u8) -> Result<Self, ElfError> {
        match st_bind {
            STB_LOCAL => Ok(Self::Local),
            STB_GLOBAL => Ok(Self::Global),
            STB_WEAK => Ok(Self::Weak),
            STB_LOPROC => Ok(Self::Loproc),
            STB_HIPROC => Ok(Self::Hiproc),
            _ => Err(ElfError::InvalidSymbolInfoBind(st_bind)),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum SymbolType {
    Notype,
    Object,
    Func,
    Section,
    File,
    Loproc,
    Hiproc,
}

impl SymbolType {
    fn from_st_type(st_type: u8) -> Result<Self, ElfError> {
        match st_type {
            STT_NOTYPE => Ok(Self::Notype),
            STT_OBJECT => Ok(Self::Object),
            STT_FUNC => Ok(Self::Func),
            STT_SECTION => Ok(Self::Section),
            STT_FILE => Ok(Self::File),
            STT_LOPROC => Ok(Self::Loproc),
            STT_HIPROC => Ok(Self::Hiproc),
            _ => Err(ElfError::InvalidSymbolInfoType(st_type)),
        }
    }
}

#[derive(Debug)]
pub struct SymbolInfo {
    st_type: SymbolType,
    st_bind: SymbolBind,
}

impl SymbolInfo {
    fn from_symbol(symbol: &Symbol) -> Result<Self, ElfError> {
        let st_type = SymbolType::from_st_type(symbol.st_symtype())?;
        let st_bind = SymbolBind::from_st_bind(symbol.st_bind())?;
        Ok(Self { st_type, st_bind })
    }
}

#[derive(Debug)]
pub enum SymbolSection {
    Undef,
    Loreserve,
    /// In the range SHN_LOPROC..=SHN_HIPROC
    Proc(u16),
    Abs,
    Common,
    Hireserve,
    /// Regular named section (.e.g .text)
    Named(String),
}

pub const SHN_LORESERVE: u16 = 0xff00;
pub const SHN_LOPROC: u16 = 0xff00;
pub const SHN_HIPROC: u16 = 0xff1f;
pub const SHN_ABS: u16 = 0xfff1;
pub const SHN_HIRESERVE: u16 = 0xffff;

impl SymbolSection {
    fn from_symbol(
        symbol: &Symbol,
        elf_file: &ElfFile,
    ) -> Result<Self, ElfError> {
        let index = symbol.st_shndx;
        let section = match index {
            SHN_UNDEF => Self::Undef,
            SHN_LORESERVE => Self::Loreserve,
            SHN_LOPROC..=SHN_HIPROC => Self::Proc(index),
            SHN_ABS => Self::Abs,
            SHN_COMMON => Self::Common,
            SHN_HIRESERVE => Self::Hireserve,
            section_index => {
                let (section_headers, string_table) =
                    elf_file.elf_bytes()?.section_headers_with_strtab()?;
                let section_headers = section_headers
                    .ok_or(ElfError::MissingSectionHeaderTable)?;
                let string_table =
                    string_table.ok_or(ElfError::MissingStringTable)?;
                let header =
                    section_headers.get(section_index.try_into().unwrap())?;
                let section_name_index = header.sh_name;
                let section_name = string_table
                    .get(section_name_index.try_into().unwrap())?
                    .to_string();
                Self::Named(section_name)
            }
        };
        Ok(section)
    }
}

#[derive(Debug)]
pub struct FullSymbol {
    pub name: Option<String>,
    section: SymbolSection,
    info: SymbolInfo,
    pub value: u32,
}

impl FullSymbol {
    fn is_global(&self) -> bool {
        self.info.st_bind == SymbolBind::Global
    }

    pub fn is_func(&self) -> bool {
        self.info.st_type == SymbolType::Func
    }

    pub fn is_section(&self) -> bool {
        self.info.st_type == SymbolType::Section
    }

    fn from_symbol(
        symbol: &Symbol,
        elf_file: &ElfFile,
    ) -> Result<Self, ElfError> {
        let elf_bytes = elf_file.elf_bytes()?;
        // Final .1 ignores the symbol table.
        let symbol_string_table = elf_bytes
            .symbol_table()?
            .ok_or(ElfError::MissingSymbolTable)?
            .1;
        let name = if symbol.st_name != 0 {
            Some(
                symbol_string_table
                    .get(symbol.st_name.try_into().unwrap())?
                    .to_string(),
            )
        } else {
            None
        };

        let section = SymbolSection::from_symbol(symbol, elf_file)?;
        let info = SymbolInfo::from_symbol(&symbol)?;
        let value = symbol.st_value.try_into().unwrap();
        Ok(Self {
            name,
            section,
            info,
            value,
        })
    }
}

struct ElfFile {
    file_data: Vec<u8>,
}

impl From<io::Error> for ElfError {
    fn from(e: io::Error) -> Self {
        Self::CouldNotOpenFile(e.to_string())
    }
}

impl From<ParseError> for ElfError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e.to_string())
    }
}

impl ElfFile {
    fn from_file(file_path: &String) -> Result<Self, ElfError> {
        let path = std::path::PathBuf::from(file_path);
        let file_data = std::fs::read(path)?;
        Ok(Self { file_data })
    }

    fn elf_bytes(&self) -> Result<ElfBytes<AnyEndian>, ElfError> {
        let slice = self.file_data.as_slice();
        let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(slice)?;
        Ok(elf_bytes)
    }

    fn string_table(&self) -> Result<StringTable, ElfError> {
        // The duplication between this function and symbol table is
        // not good. Also note the error returned is "missing symbol
        // table". To fix.
        let elf_bytes = self.elf_bytes()?;
        // Final .1 ignores the symbol table.
        let strtab = elf_bytes
            .symbol_table()?
            .ok_or(ElfError::MissingSymbolTable)?
            .1;
        Ok(strtab)
    }

    /// This function has a bug -- string table returned is tied to
    /// symbol table.
    fn symbol_table(&self) -> Result<SymbolTable<AnyEndian>, ElfError> {
        let elf_bytes = self.elf_bytes()?;
        // Final .0 ignores the string table.
        let symtab = elf_bytes
            .symbol_table()?
            .ok_or(ElfError::MissingSymbolTable)?
            .0;
        Ok(symtab)
    }

    fn section_header_table(
        &self,
    ) -> Result<SectionHeaderTable<AnyEndian>, ElfError> {
        let elf_bytes = self.elf_bytes()?;
        let section_header_table = elf_bytes
            .section_headers_with_strtab()?
            .0 // Ignore the string table
            .ok_or(ElfError::MissingSectionHeaderTable)?;
        Ok(section_header_table)
    }

    fn segments(&self) -> Result<SegmentTable<AnyEndian>, ElfError> {
        let elf_bytes = self.elf_bytes()?;
        if let Some(segment_table) = elf_bytes.segments() {
            Ok(segment_table)
        } else {
            Err(ElfError::MissingSegmentTable)
        }
    }

    /// Get the data in the section corresponding to a program header
    fn segment_data(&self, header: &ProgramHeader) -> Result<&[u8], ElfError> {
        let elf_bytes = self.elf_bytes()?;
        let data = elf_bytes.segment_data(header);
        data.map_err(|e| ElfError::ParseError(e.to_string()))
    }

    /// Get the data in the section corresponding to a section header
    fn section_data(&self, header: &SectionHeader) -> Result<&[u8], ElfError> {
        let elf_bytes = self.elf_bytes()?;
        let data_pair = elf_bytes.section_data(header)?;
        if data_pair.1.is_some() {
            unimplemented!("found unexpected compression in section")
        }
        Ok(data_pair.0)
    }

    /// Returns the list of global or function symbols (other symbols
    /// are ignored).
    fn symbols(&self) -> Result<Vec<FullSymbol>, ElfError> {
        let symtab = self.symbol_table()?;

        let mut symbols = Vec::new();
        for entry in symtab.iter() {
            let symbol = FullSymbol::from_symbol(&entry, self)?;
            if symbol.is_global() || symbol.is_func() {
                symbols.push(symbol)
            }
        }
        Ok(symbols)
    }
}

pub trait ElfLoadable {
    /// Write a byte of data to an address in the elf-loadable target
    fn write_byte(&mut self, addr: u32, data: u8) -> Result<(), ElfError>;

    /// Load the symbols in the elf file, as a map from symbol names
    /// to symbol values. Can be implemented as ignoring symbol_map if
    /// symbols are not required.
    fn load_symbols(&mut self, symbols: Vec<FullSymbol>);
}

fn alloc(section_flags: u64) -> bool {
    section_flags & u64::from(SHF_ALLOC) != 0
}

fn write(section_flags: u64) -> bool {
    section_flags & u64::from(SHF_WRITE) != 0
}

/// Read an ELF file from disk and load the alloc section (the ones
/// meant to be present during program execution) into memory. Prints
/// what it is doing.
pub fn load_elf<L: ElfLoadable>(
    loadable: &mut L,
    elf_file_path: &String,
) -> Result<(), ElfError> {
    let elf_file = ElfFile::from_file(elf_file_path)?;
    let segments = elf_file.segments()?;

    for program_header in segments.iter() {
        let data = elf_file.segment_data(&program_header)?;
        let section_load_address = program_header.p_paddr;
        println!("{:x?}", program_header);

        for (offset, byte) in data.iter().enumerate() {
            let addr = section_load_address + u64::try_from(offset).unwrap();
            loadable.write_byte(addr.try_into().unwrap(), (*byte).into())?;
        }
    }

    loadable.load_symbols(elf_file.symbols()?);
    Ok(())
}
