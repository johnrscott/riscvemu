use itertools::{Itertools, PeekingNext};
use riscvemu::decode::Decoder;
use riscvemu::platform::{Instr, Platform};
use riscvemu::platform::arch::{make_rv32i, make_rv32m, make_rv32zicsr, make_rv32priv};
use riscvemu::utils::mask;
use riscvemu::elf_utils::{load_elf, ElfLoadable, ElfLoadError};
use std::collections::BTreeMap;
use std::fs::File;
use std::io::{prelude::*, BufReader, LineWriter, Lines};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum TraceFileError {
    #[error("missing section heading at start of file")]
    MissingSectionHeading,
    #[error("section .eeprom is compulsory")]
    MissingEepromSection,
    #[error("section {0} is not recognised/implemented")]
    UnrecognisedSection(String),
}

pub type Result<T> = std::result::Result<T, TraceFileError>;

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

#[derive(Debug, Clone)]
enum Section {
    Eeprom(BTreeMap<u32, u32>),
}

impl ElfLoadable for Section {
    fn write_byte(
        &mut self,
        addr: u32,
        data: u8,
    ) -> std::result::Result<(), ElfLoadError> {
	let aligned_addr = 0xffff_fffc & addr;
	let offset = addr - aligned_addr;
	match self {
	    Section::Eeprom(map) => {
		let instr_part = u32::from(data) << 8*offset;
		if let Some(instr) = map.get_mut(&aligned_addr) {
		    *instr |= instr_part;	    
		} else {
		    map.insert(aligned_addr, instr_part);
		}
	    }
	    _ => unimplemented!("Cannot load non-eeprom section from elf")
	}
	Ok(())
    }
}

fn write_section(file: &mut LineWriter<File>, section: Section) {

    let mut decoder = Decoder::<Instr<Platform>>::new(mask(7));
    make_rv32i(&mut decoder).expect("adding instructions should work");
    make_rv32m(&mut decoder).expect("adding instructions should work");
    make_rv32zicsr(&mut decoder).expect("adding instructions should work");
    make_rv32priv(&mut decoder).expect("adding instructions should work");

    match section {
        Section::Eeprom(map) => {
            file.write_all(b".eeprom\n").expect("should work");
            for (addr, instr) in map.into_iter() {
		let Instr { printer, .. } = decoder.get_exec(instr).unwrap();
		let asm = printer(instr);
                file.write_all(
                    format!("{addr:0>8x}  {instr:0>8x}  # {asm}\n")
                        .as_bytes(),
                )
                .expect("should write")
            }
        }
        _ => unimplemented!("Not yet implemented writing that section"),
    }
}

fn read_section<I>(lines: &mut I) -> Result<Section>
where
    I: Iterator<Item = String> + PeekingNext,
{
    if let Some(first_line) = lines.next() {
        match first_line.as_ref() {
            ".eeprom" => {
                let eeprom = lines
                    .peeking_take_while(|line| !is_section_header(line))
                    .map(get_addr_instr_tuple)
                    .collect();
                Ok(Section::Eeprom(eeprom))
            }
            _ => Err(TraceFileError::UnrecognisedSection(first_line)),
        }
    } else {
        Err(TraceFileError::MissingSectionHeading)
    }
}

fn main() {
    let file = File::open("out.trace").expect("file should exist");
    let reader = BufReader::new(file);

    let mut iter = reader
        .lines()
        .flatten() // note this drops line errors
        .map(get_non_comment)
        .flatten()
        .peekable();
    while iter.peek().is_some() {
        match read_section(&mut iter) {
            Ok(section) => {
                println!("{section:?}");
                //sections.push(section);
            }
            Err(e) => match e {
                TraceFileError::UnrecognisedSection(name) => {
                    println!("Warning: unrecognised section {name}")
                }
                _ => panic!("Error {e} occurred"),
            },
        }
    }

    let mut section = Section::Eeprom(BTreeMap::new());
    let elf_name = "c/hello.out".to_string();
    load_elf(&mut section, &elf_name);

    let file = File::create("out.trace").expect("should be able to write");
    let mut file = LineWriter::new(file);
    write_section(&mut file, section)
}
