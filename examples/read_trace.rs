use std::collections::HashMap;
use std::fs::File;
use std::io::{prelude::*, BufReader};

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

fn main() {
    let file = File::open("test.trace").expect("file should exist");
    let reader = BufReader::new(file);

    let eeprom: HashMap<u32, u32> = reader
        .lines()
        .flatten() // note this drops line errors
        .map(get_non_comment)
        .flatten()
	// This currently ignores section
        .filter(|non_comment| !is_section_header(&non_comment))
        .map(get_addr_instr_tuple)
        .collect();

    println!("{:?}", eeprom);
}
