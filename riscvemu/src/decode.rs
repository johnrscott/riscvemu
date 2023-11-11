use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecodeError {
    #[error("missing next step for instruction 0x{instr:x} using mask 0x{mask:x}")]
    MissingNextStep { instr: u32, mask: u32 },
}

pub fn test() {
    println!("Executed!");
}

/// Next step in the decoding process
///
/// This is not the first step; the first step is never
/// an execution function, because decoding based on at
/// least the opcode is always required first.
enum NextStep {
    Decode(Decoder),
    Exec(fn() -> ()),
}

/// Contains information required to decode an instruction
///
/// Decoding happens in multiple steps, each of which involves masking
/// out a portion of the function and then comparing the result with a
/// set of values. Depending on the value obtained, decoding proceeds
/// to the next step. The next step may either be another Decoder, or
/// a function that can be used to execute the function.
///
/// The mask can be used to pick out first the opcode, then funct3 or
/// funct7, or any other fields required for decoding.
///
pub struct Decoder {
    mask: u32,
    value_map: HashMap<u32, NextStep>,
}

impl Decoder {
    fn next_step(&self, instr: u32) -> Result<&NextStep, DecodeError> {
        let value = self.mask & instr;
        if let Some(next_step) = self.value_map.get(&value) {
            Ok(next_step)
        } else {
            Err(DecodeError::MissingNextStep {
                instr,
                mask: self.mask,
            })
        }
    }

    pub fn get_exec(&self, instr: u32) -> Result<fn() -> (), DecodeError> {
        match self.next_step(instr)? {
            NextStep::Decode(decoder) => decoder.get_exec(instr),
            NextStep::Exec(exec) => Ok(*exec),
        }
    }
}
