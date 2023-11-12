use std::collections::{hash_map::Entry, HashMap};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DecoderError {
    #[error("missing next step for mask 0x{mask:x} and value 0x{value:x}")]
    MissingNextStep { mask: u32, value: u32 },
    #[error("resulting decoder would have an ambiguous mask")]
    AmbiguousMask,
    #[error("resulting decoder would have an ambiguous next step following value")]
    AmbiguousNextStep,
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
    /// vector will get inserted into decoder first. This is because it
    /// is easier to remove items from the end of a vector (for the recursion)
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
pub struct MaskWithValue {
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
    /// The masks_with_values vector is in reverse order; items at the end
    /// come first in the decoding process.
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
    /// If exactly the same masks_with_values vector is supplied multiple
    /// times, that is not an error; the exec function will be written each
    /// time with whatever you pass as exec. You can decide if you think this
    /// is a feature or not.
    ///
    /// The decoder that results from adding instructions cannot be ambiguous.
    /// Ambiguities decoding can arise through one of the following
    /// mechanisms:
    /// - the decoder is not sure what mask to apply next. This is
    ///   called an ambiguous mask error.
    /// - the decoder does not know whether the next step following a
    ///   value is an execution function or another decoder. This
    ///   is called an ambiguous next step error.
    ///
    /// Whether a decoding process will result in one of these ambiguities
    /// is known when this function attempts to add the instruction to
    /// the decoder, and results in returning an error variant for the
    /// corresponding error.
    ///
    /// If an error variant is returned, the function has the strong
    /// exception guarantee that the state of the decoder has not changed.
    ///
    pub fn push_instruction(
        &mut self,
        mut masks_with_values: Vec<MaskWithValue>,
        exec: fn() -> (),
    ) -> Result<(), DecoderError> {

	// Check if at least on mask/value is given -- this is
	// required because with no mask or value, there is nothing
	// the decoder can do to check the instruction.
        if masks_with_values.len() == 0 {
            return Err(DecoderError::NoDecodingMaskSpecified);
        }

	// The gist of the algorithm is to traverse the tree along
	// the branch specified by mask/value pairs, checking that
	// part of the branch is consistent with the mask/value vector.
	// Then, upon reaching a node with the value missing, use
	// the tail part of the mask/value vector to construct the
        // remaining part of the branch, with the exec function at
	// the end, and append it to this node.
	//
	// Most of the function does the first part. The creating
	// and attaching the tail of the branch happens just after
	// the loop {} below finishes.
	
	// Begin with the route of the tree. The decoder variable
	// will successively point to nodes moving down the branch
	// specified by the masks/values
        let mut decoder = self;

	// Make an iterator which successively pops from the end
	// of the masks/values vector (peekable means you can see
	// whether the value after the current one is there or not)
	let mut it = masks_with_values.drain(0..).peekable();

        // Starting at the end of the vector, successively remove
        // items one by one, checking that they are consistent
        // with the tree structure of the decoder
	let new_value = loop {

	    // Get the current mask and value (popping from the end of vector)
	    if let Some(MaskWithValue { mask, value }) = it.next() {

		// Check the mask is compatible with the decoder (i.e.
		// the mask in this node matches mask)
		if !decoder.mask_matches(&mask) {
                    return Err(DecoderError::AmbiguousMask);
		}
		
		// Check if the value is present in the map for this node
		if decoder.contains_value(&value) {
                    // If the value is in the map, then this mask/value pair
                    // is compatible with the tree. Get the next step.
                    let next_step = decoder
			.value_map
			.get_mut(&value)
			.expect("it is present, because we just checked");

                    // If the next step is an execution function, but there are
                    // still items left in masks_with_values, then there will be
                    // a next-step ambiguity. Return error.
                    decoder = match next_step {
			NextStep::Decode(d) => d,
			NextStep::Exec(e) => {
			    if !it.peek().is_none() {
				return Err(DecoderError::AmbiguousNextStep)
			    } else {
				// Otherwise, if the length of masks
				// with values is zero (there are no
				// more values to pop), then we are in
				// fact re-inserting the same decoder
				// as was previously inserted, with a
				// new exec function. Write this new
				// exec function and exit.
				*e = exec;
				return Ok(())
			    }
			}
		    }
		} else {
		    // If, on the other hand, the value is not present in the
		    // decoder, then break here. At this point, it is time to
		    // add the tail of the branch on to the decoder. Return the
		    // value for use outside the loop (as new_value)
		    break value;
		}

	    } else {
		// If there are no more masks/values left, then we have
		// walked a branch where every mask agreed and every value
		// had a decoder in the map, including the final one (which
		// pointed to the decoder stored in the decoder variable).
		// This is an error, because the current masks_and_values would
		// introduce an ambiguity with what is already in the tree. The
		// error is an ambiguous-next-step error, because the current
		// next step is a decoder, but the current masks_and_values
		// implies it should be an exec function.
		return Err(DecoderError::AmbiguousNextStep);
	    }
        };

	// The state at this point is that decoder points to some node in
	// the decode tree, and it is an iterator which contains the remaining
	// items that will form the tail of the branch starting at this node,
	// and exec is the function which should be placed at the leaf.
	let tail_masks_with_values = it.collect();
	let next_step = NextStep::from_masks_with_values(tail_masks_with_values, exec);
	decoder.value_map.insert(new_value, next_step);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn 
    
}
