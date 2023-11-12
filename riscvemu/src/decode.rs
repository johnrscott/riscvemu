use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("missing next step for instruction 0x{instr:x} using mask 0x{mask:x}")]
    MissingNextStep { instr: u32, mask: u32 },
    #[error("attempt to add ambiguous decoding (conflicting mask 0x{mask:x}")]
    AmbiguousDecodingMask { mask: u32 },
    #[error("attempt to add decoding conflicting with existing exec function")]
    ConflictingExec,
    #[error("missing first mask")]
    MissingFirstMask,
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
    pub fn exec_step(exec: fn() -> ()) -> Self {
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
        masks_with_values: Vec<MaskWithValue>,
        exec: fn() -> (),
    ) -> Result<(), DecoderError> {
        let mut decoder = self;
        let mut masks_with_values_iter = masks_with_values.iter();

        // Get the first list value
        let mut mask_with_value = masks_with_values_iter.next();
        if let None = mask_with_value {
            return Err(DecoderError::MissingFirstMask);
        }

        loop {
            let MaskWithValue { mask, value } = mask_with_value.expect("must be present else bug");

            // Check mask matches
            if decoder.mask != *mask {
                return Err(DecoderError::AmbiguousDecodingMask { mask: *mask });
            }
            // Check value is present in map
            if let Some(next_step) = decoder.value_map.get_mut(&value) {
                match next_step {
                    NextStep::Decode(next_decoder) => decoder = next_decoder,
                    NextStep::Exec(_) => return Err(DecoderError::ConflictingExec),
                }

                mask_with_value = masks_with_values_iter.next();
                if let None = mask_with_value {
                    break;
                }
            } else {
                // There is no map entry for this value. That means that
                // we have reached the node in the tree (decoder) where
                // the new branch needs to be added.
                break;
            }
        }

        // Now, decoder is a node in the tree containing a map that does not
        // contain value. The iterator is at the position where its mask matches
	// decoder, but decoder.value_map does not contain the value, or,
	// the iterator is None, meaning 

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
