use crate::decode::Decoder;
use crate::elf_utils::{load_elf, ElfError, ElfLoadable, FullSymbol};
use crate::platform::arch::{
    make_rv32i, make_rv32m, make_rv32priv, make_rv32zicsr,
};
use crate::platform::{Instr, Platform};
use crate::utils::mask;
use itertools::{Itertools, PeekingNext};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, prelude::*, BufReader, LineWriter};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraceFileError {
    #[error("missing section heading at start of file")]
    MissingSectionHeading,
    #[error("section .eeprom is compulsory")]
    MissingEepromSection,
    #[error("section {0} is not recognised/implemented")]
    UnrecognisedSection(String),
    #[error("error parsing entry {0} in .trace section")]
    ParseTraceSectionFailed(String),
    #[error("error processing ELF file: {0}")]
    ElfError(ElfError),
    #[error("Trace file I/O error: {0}")]
    IoError(String),
}

impl From<ElfError> for TraceFileError {
    fn from(e: ElfError) -> Self {
        Self::ElfError(e)
    }
}

impl From<io::Error> for TraceFileError {
    fn from(e: io::Error) -> Self {
        Self::IoError(e.to_string())
    }
}

/// If the line ends in a comment, remove it. If the
/// result contains any non-whitespace characters,
/// return it as Some. Otherwise, return None. (Covers
/// empty lines and comment-only lines).
fn get_non_comment(line: String) -> Option<String> {
    let without_comment = &line[0..line.find("#").unwrap_or(line.len())];
    if without_comment.trim().is_empty() {
        None
    } else {
        Some(without_comment.to_string())
    }
}

/// Return true if the line begins with a dot (.)
fn is_section_header(line: &String) -> bool {
    line.chars().nth(0).unwrap() == '.'
}

fn get_addr_instr_tuple(non_comment_line: String) -> (u32, u32) {
    let terms: Vec<u32> = non_comment_line
        .split_whitespace()
        .into_iter()
        .map(|term| u32::from_str_radix(term, 16))
        .map(|res| res.expect("term should be hex"))
        .collect();
    if terms.len() != 2 {
        panic!("Line length should be 2")
    }
    let addr = terms[0];
    let instr = terms[1];
    (addr, instr)
}

fn parse_dec_or_hex(value: &str) -> Result<u32, TraceFileError> {
    if value.len() > 2 && &value[0..2] == "0x" {
	// Value is hexadecimal
	u32::from_str_radix(&value[2..], 16)
    } else {
	value.parse()
    }.map_err(|_| TraceFileError::ParseTraceSectionFailed(value.to_string()))
}

fn get_trace_key_value_tuple(
    non_comment_line: String,
) -> Result<Property, TraceFileError> {
    let (key, value) = non_comment_line.split_once(char::is_whitespace).ok_or(
        TraceFileError::ParseTraceSectionFailed(non_comment_line.to_string()),
    )?;
    let value = value.trim();
    if key == "pc" {
        let pc = parse_dec_or_hex(value)?;
        Ok(Property::Pc(pc))
    } else if key.starts_with("x") {
        // Remove first character and interpret number as register
        // index
        let mut chars = key.chars();
        chars.next();
        let reg_index_string: String = chars.collect();
        let index: u8 = reg_index_string.parse().map_err(|_| {
            TraceFileError::ParseTraceSectionFailed(value.to_string())
        })?;
        let value = parse_dec_or_hex(value)?;
        Ok(Property::Reg { index, value })
    } else if key == "uart" {
        // Remove first and last quotes/speech marks characters
        // Note: no check is performed.
        let value = value.replace("\\n", "\n");
        let mut chars = value.chars();
        chars.next();
        chars.next_back();
        Ok(Property::Uart(chars.collect()))
    } else {
        Err(TraceFileError::ParseTraceSectionFailed(value.to_string()))
    }
}

#[derive(Debug, Error)]
pub enum TraceCheckFailed {
    #[error("Cannot advance to cycle {required} from current value {current}")]
    CannotAdvanceToCycle { current: u64, required: u64 },
    #[error(
        "Expected property {expected:?}, found {found:?} at cycle {cycle}"
    )]
    FailedCheck {
        cycle: u64,
        expected: Property,
        found: Property,
    },
}

/// Check where a property is satisfied at a particular clock cycle
pub trait TraceCheck {
    /// Check whether a property is satisfied
    ///
    /// This function advances to the specified cycle and then checks
    /// the expected property is satisfied. If it is, Ok is returned;
    /// otherwise, the reason for failure is returned
    fn check_trace_point(
        &mut self,
        trace_point: TracePoint,
    ) -> Result<(), TraceCheckFailed>;
}

#[derive(Debug)]
pub enum Property {
    /// Value of the program counter
    Pc(u32),
    /// The state of a register x{index} should be {value}
    Reg { index: u8, value: u32 },
    /// The UART buffer should have received this string
    Uart(String),
}

/// This defines a check that should occur at a particular clock cycle
///
/// The clock cycle corresponds to the mcycle 64-bit register. If the
/// device being modelled takes action on a rising clock edge
/// (i.e. registers and flip-flops are rising-edge-triggered), then
/// the state of the device under test is tested on clock falling
/// edges. The mcycle register contains the number of rising edges
/// that have occurred at this point since reset. Note that any
/// computation performed on a rising edge of the clock uses the value
/// of mcycle at the previous falling edge.
///
/// The properties to be tested are stored in the properties item.
#[derive(Debug)]
pub struct TracePoint {
    pub cycle: u64,
    pub properties: Vec<Property>,
}

#[derive(Debug)]
pub enum Section {
    Eeprom {
        section_data: BTreeMap<u32, u32>,
        symbols: Vec<FullSymbol>,
    },
    Trace(TracePoint),
}

impl Section {
    fn new_eeprom() -> Self {
        Self::Eeprom {
            section_data: BTreeMap::new(),
            symbols: Vec::new(),
        }
    }
}

impl ElfLoadable for Section {
    fn write_byte(
        &mut self,
        addr: u32,
        data: u8,
    ) -> std::result::Result<(), ElfError> {
        let aligned_addr = 0xffff_fffc & addr;
        let offset = addr - aligned_addr;
        match self {
            Section::Eeprom { section_data, .. } => {
                let instr_part = u32::from(data) << 8 * offset;
		let mask = !(mask(8) << 8*offset);
                if let Some(instr) = section_data.get_mut(&aligned_addr) {
                    *instr = (mask & *instr) | instr_part;
                } else {
                    section_data.insert(aligned_addr, instr_part);
                }
            }
            _ => unimplemented!("Cannot load non-eeprom section from elf"),
        }
        Ok(())
    }

    fn load_symbols(&mut self, new_symbols: Vec<FullSymbol>) {
        match self {
            Section::Eeprom { symbols, .. } => *symbols = new_symbols,
            _ => unimplemented!("cannot symbols to non-eeprom section"),
        }
    }
}

fn get_symbol_at_address(
    addr: u32,
    symbols: &Vec<FullSymbol>,
) -> Option<&FullSymbol> {
    symbols.iter().find(|&symbol| symbol.value == addr)
}

fn write_section(file: &mut LineWriter<File>, section: Section) {
    let mut decoder = Decoder::<Instr<Platform>>::new(mask(7));
    make_rv32i(&mut decoder).expect("adding instructions should work");
    make_rv32m(&mut decoder).expect("adding instructions should work");
    make_rv32zicsr(&mut decoder).expect("adding instructions should work");
    make_rv32priv(&mut decoder).expect("adding instructions should work");

    match section {
        Section::Eeprom {
            section_data,
            symbols,
        } => {
            file.write_all(b".eeprom\n").expect("should work");
            for (addr, instr) in section_data.into_iter() {
                // Check for a function label at this address
                if let Some(symbol) = get_symbol_at_address(addr, &symbols) {
                    let name = symbol.name.clone().unwrap();
                    file.write_all(format!("\n# {name}\n").as_bytes())
                        .expect("should write");
                }

                let asm = if let Ok(Instr { printer, .. }) =
                    decoder.get_exec(instr)
                {
                    printer(instr)
                } else {
                    "unknown/not instruction".to_string()
                };
                file.write_all(
                    format!("{addr:0>8x}  {instr:0>8x}  # {asm}\n").as_bytes(),
                )
                .expect("should write")
            }
        }
        _ => unimplemented!("Not yet implemented writing that section"),
    }
}

fn read_section<I>(lines: &mut I) -> Result<Section, TraceFileError>
where
    I: Iterator<Item = String> + PeekingNext,
{
    if let Some(first_line) = lines.next() {
        if first_line == ".eeprom" {
            let section_data = lines
                .peeking_take_while(|line| !is_section_header(line))
                .map(get_addr_instr_tuple)
                .collect();
            Ok(Section::Eeprom {
                section_data,
                symbols: Vec::new(),
            })
        } else if first_line.starts_with(".trace") {
            let cycle: u64 = first_line
                .strip_prefix(".trace.")
                .expect("prefix is present")
                .parse()
                .map_err(|_| {
                    TraceFileError::ParseTraceSectionFailed(first_line)
                })?;
            let properties_result: Result<Vec<_>, _> = lines
                .peeking_take_while(|line| !is_section_header(line))
                .map(get_trace_key_value_tuple)
                .collect();
            let properties = properties_result?;
            let trace_point = TracePoint { cycle, properties };
            Ok(Section::Trace(trace_point))
        } else {
            Err(TraceFileError::UnrecognisedSection(first_line))
        }
    } else {
        Err(TraceFileError::MissingSectionHeading)
    }
}

pub trait TraceLoadable {
    fn push(&mut self, section: &Section);
}

/// Load a trace file from file. Returns the set of trace points (the
/// sections that are not the .eeprom section)
pub fn load_trace<L: TraceLoadable>(
    loadable: &mut L,
    trace_file_path: String,
) -> Result<Vec<TracePoint>, TraceFileError> {
    let file = File::open(trace_file_path)?;
    let reader = BufReader::new(file);

    let mut trace_points = Vec::new();

    let mut iter = reader
        .lines()
        .flatten() // note this drops line errors
        .map(get_non_comment)
        .flatten()
        .peekable();
    while iter.peek().is_some() {
        match read_section(&mut iter) {
            Ok(section) => match section {
                Section::Eeprom { .. } => loadable.push(&section),
                Section::Trace(trace_point) => trace_points.push(trace_point),
            },
            Err(e) => match e {
                TraceFileError::UnrecognisedSection(name) => {
                    println!("Warning: unrecognised section {name}")
                }
                _ => panic!("Error {e} occurred"),
            },
        }
    }

    let in_cycle_order = trace_points
        .into_iter()
        .sorted_by_key(|trace_point| trace_point.cycle)
        .collect();

    Ok(in_cycle_order)
}

pub fn elf_to_trace_file(
    elf_path_in: String,
    trace_path_out: String,
) -> Result<(), TraceFileError> {
    let mut section = Section::new_eeprom();
    load_elf(&mut section, &elf_path_in)?;

    let file = File::create(trace_path_out)?;
    let mut file = LineWriter::new(file);
    write_section(&mut file, section);
    Ok(())
}
