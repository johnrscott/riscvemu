use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("missing next step for mask 0x{mask:x} and value 0x{value:x}")]
    MissingNextStep { mask: u32, value: u32 },
    #[error("attempt to add ambiguous decoding (conflicting mask 0x{mask:x}")]
    AmbiguousDecodingMask { mask: u32 },
    #[error("attempt to add decoding conflicting with existing exec function")]
    ConflictingExec,
    #[error("at least one decoder and value is compulsory in push_instruction")]
    NoDecodingMaskSpecified,
}

pub fn test() {
    println!("Executed!");
}

/// Next step in the decoding process
///
/// This is not the first step; the first step is never
/// an execution function, because decoding based on at
/// least the opcode is always required first.
#[derive(Debug)]
enum NextStep {
    Decode(Decoder),
    Exec(fn() -> ()),
}

impl NextStep {

    /// masks_with_values is in reverse order; values at the end of the
    /// vector will get inserted into decoder first
    fn from_masks_with_values(mut masks_with_values: Vec<MaskWithValue>, exec: fn() -> ()) -> Self {
        let length = masks_with_values.len();
        if length == 0 {
            Self::Exec(exec)
        } else {
            let mut value_map = HashMap::new();
            // Get the last element and drop it from the vector
            let MaskWithValue { mask, value } = masks_with_values
                .drain(length - 1..)
                .next()
                .expect("since the vector has at least one element, this will work");
            // Get the next step, which recursively constructs all the next steps
            // all the way down to the end
            let next_step = Self::from_masks_with_values(masks_with_values, exec);
            value_map.insert(value, next_step);
            let decoder = Decoder { mask, value_map };
            Self::Decode(decoder)
        }
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
#[derive(Debug, Default)]
pub struct Decoder {
    mask: u32,
    value_map: HashMap<u32, NextStep>,
}

/// Represents a node and subsequent edge in the decoder tree
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

    fn next_step_for_value(&self, value: &u32) -> Result<&NextStep, DecoderError> {
        if let Some(next_step) = self.value_map.get(value) {
            Ok(next_step)
        } else {
            Err(DecoderError::MissingNextStep {
                mask: self.mask,
                value: *value,
            })
        }
    }

    fn mask_matches(&self, mask: &u32) -> bool {
        self.mask == *mask
    }

    fn contains_value(&self, value: &u32) -> bool {
        self.value_map.contains_key(value)
    }

    fn is_consistent(&self, mask_with_value: &MaskWithValue) -> bool {
        let MaskWithValue { mask, value } = mask_with_value;
        self.mask_matches(mask) && self.contains_value(value)
    }

    /// Get the next step by applying mask to instruction and checking value
    fn next_step_for_instr(&self, instr: u32) -> Result<&NextStep, DecoderError> {
        let value = self.mask & instr;
        self.next_step_for_value(&value)
    }

    pub fn get_exec(&self, instr: u32) -> Result<fn() -> (), DecoderError> {
        match self.next_step_for_instr(instr)? {
            NextStep::Decode(decoder) => decoder.get_exec(instr),
            NextStep::Exec(exec) => Ok(*exec),
        }
    }

    /// Add an instruction, specified by a sequence of masks and expected values
    ///
    /// The list of mask/value pairs is what will be used to identify the
    /// instruction. Each mask is checked in turn, with a matching value
    /// meaning the decoder will use the next mask and look for the next
    /// value, continuing until the execution function is reached.
    ///
    /// The decoder is a tree, where each node contains a mask that will
    /// be applied, and each edge contains a value that can be obtained
    /// from this mask. Decoding an instruction means following a branch
    /// from the root mask to a leaf, which holds the function to execute
    /// the instruction.
    ///
    /// Adding an instruction to the decoder amounts to adding a new branch
    /// to the tree.
    ///
    /// The masks_with_vector vector must contain at least one item,
    /// otherwise an error variant is returned.
    ///
    /// When inserting into an already populated decoder tree, two
    /// errors can happen:
    /// 1. the mask following a value can conflict with the mask already
    ///    in the tree. This represents a decoding ambiguity, because the
    ///    decoder will not be able to decide which mask to apply next,
    ///    and causes an error variant to be returned here.
    /// 2. the branch being added to the tree may be a subset of
    ///    an already existing branch. In this case, either the same
    ///    instruction is already present, or the new decoding is not
    ///    possible because it already decodes an already existing
    ///    instruction
    ///
    /// If an error variant is returned, the function has the strong
    /// exception guarantee that the state of the decoder has not changed.
    ///
    pub fn push_instruction(
        &mut self,
        masks_with_values: Vec<MaskWithValue>,
        exec: fn() -> (),
    ) -> Result<(), DecoderError> {
        if masks_with_values.len() == 0 {
            return Err(DecoderError::NoDecodingMaskSpecified);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn check_next_step_from_masks_with_values() {

	fn execute() { println!("Exec")}
	
	let mv1 = MaskWithValue { mask: 0x1, value: 0x2 };
	let mv2 = MaskWithValue { mask: 0x3, value: 0x4 };
	let mv3 = MaskWithValue { mask: 0x5, value: 0x5 };
	let masks_with_values = vec![mv3, mv2, mv1];
	let next_steps = NextStep::from_masks_with_values(masks_with_values, execute);
	println!("{next_steps:?}");
	
	assert_eq!(0,1);
	
    }

}
