use std::collections::{HashMap, hash_map::Entry};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
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

impl NextStep {
    pub fn exec_step(exec: fn() -> ()) -> Self{
	Self::Exec(exec)
    }

    pub fn decode_step(decoder: Decoder) -> Self {
	Self::Decode(decoder)
    }

    
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
#[derive(Default)]
pub struct Decoder {
    mask: u32,
    value_map: HashMap<u32, NextStep>,
}

struct MaskWithValue {
    mask: u32,
    value: u32,
}

impl Decoder {
    /// Create a new decoder
    ///
    /// Specify the top level mask. This is the first part of the
    /// instruction that will be read (in RISC-V, the opcode field).
    ///
    pub fn new(mask: u32) -> Self {
        Self {
            mask,
            ..Self::default()
        }
    }
    
    fn next_step(&self, instr: u32) -> Result<&NextStep, DecoderError> {
        let value = self.mask & instr;
        if let Some(next_step) = self.value_map.get(&value) {
            Ok(next_step)
        } else {
            Err(DecoderError::MissingNextStep {
                instr,
                mask: self.mask,
            })
        }
    }

    pub fn get_exec(&self, instr: u32) -> Result<fn() -> (), DecoderError> {
        match self.next_step(instr)? {
            NextStep::Decode(decoder) => decoder.get_exec(instr),
            NextStep::Exec(exec) => Ok(*exec),
        }
    }

    fn push_decode_step(&mut self, value: u32, next_step: NextStep) {
	self.value_map.insert(value, next_step);
    }
    
    /// Add an instruction, specified by a sequence of masks and expected values
    ///
    /// The list of mask/value pairs is what will be used to identify the
    /// instruction. Each mask is checked in turn, with a matching value
    /// meaning the decoder will use the next mask and look for the next
    /// value, continuing until the execution function is reached.
    pub fn push_instruction(
        &mut self,
	first_value: u32,
        masks_with_values: Vec<MaskWithValue>,
        exec: fn() -> (),
    ) -> Result<(), DecoderError> {	

	// Decoder (self) is a tree, where breaking into branches
	// happens at the map. Each node contains a mask. Each
	// branch represents a possible value from that mask.
	// This function needs to insert a branch from the root
	// down to the leaf (containing the exec function).

	// The root node is always present, with self.mask as the
	// mask value. From then on, branches either exist or
	// do not exist.

	// If a branch exists, proceed down it (meaning masks and
	// fields match).

	// When you get to a node or branch that doesn't exist,
	// start inserting new branches and nodes all the way down
	// to the leaf (the execution function).

	Ok(())
    }
}
