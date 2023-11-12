//! RV32I base integer instruction set
//!
//! This file holds the instructions defined in chapter 2,
//! unprivileged specification version 20191213.
//!

use super::{
    decode::{ExecFn32, Decoder},
    exec::{execute_auipc_rv32i, execute_jal_rv32i, execute_jalr_rv32i, execute_lui_rv32i},
    opcodes::{OP_AUIPC, OP_JAL, OP_JALR, OP_LUI},
};
use std::collections::{HashMap, hash_map::Entry};

/// In RV32I and RV64I, If branch is taken, set pc = pc + offset,
/// where offset is a multiple of two; else do nothing. The
/// offset is 13 bits long.
///
/// The condition for branch taken depends on the value in
/// mnemonic, which is one of:
/// - "beq": src1 == src2
/// - "bne": src1 != src2
/// - "blt": src1 < src2 as signed integers
/// - "bge": src1 >= src2 as signed integers
/// - "bltu": src1 < src2 as unsigned integers
/// - "bgeu": src1 >= src2 as unsigned integers
///
/// Only on branch-taken, an instruction-address-misaligned
/// exception is generated if the target pc is not 4-byte
/// aligned.
pub enum Branch {
    Beq,
    Bne,
    Blt,
    Bge,
    Bltu,
    Bgeu,
}

/// In RV32I and RV64I, load the data at address base + offset
/// into dest. The offset is 12 bits long.
///
/// The size of data, and the way it is loaded into dest, depends
/// on the mnemonic, as follows:
///
/// In RV32I:
/// - "lb": load a byte, sign extend in dest
/// - "lh": load a halfword, sign extend in dest
/// - "lw": load a word
/// - "lbu": load a byte, zero extend in dest
/// - "lhu": load a halfword, zero extend in dest
///
/// In RV64I:
/// - "lw": load a word, sign extend in dest
/// - "lwu": load a word, zero extend in dest
/// - "ld": load a doubleword
///
/// Loads do not need to be aligned
pub enum Load {
    Lb,
    Lh,
    Lw,
    Lbu,
    Lhu,
}

/// In RV32I and RV64I, load the data at src into address base +
/// offset. The offset is 12 bits long.
///
/// The mnemonic determines the width of data that is stored to
/// memory:
///
/// In RV32I:
/// - "sb": store a byte
/// - "sh": store a halfword
/// - "sw": store a word
///
/// In RV64I:
/// - "sd": store a doubleword
///
/// Stores do not need to be aligned
pub enum Store {
    Sb,
    Sh,
    Sw,
}

/// In RV32I and RV64I, perform an operation between the value in
/// register src and the sign-extended version of the 12-bit
/// i_immediate.
///
/// The operation performed is determined by the mnemonic as follows:
/// - "addi": dest = src + i_immediate
/// - "slti": dest = (src < i_immediate) ? 1 : 0, signed comparison
/// - "sltiu": dest = (src < i_immediate) ? 1 : 0, unsigned comparison
/// - "andi": dest = src & i_immediate
/// - "ori": dest = src | i_immediate
/// - "xori": dest = src ^ i_immediate
/// - "slli": dest = src << (0x1f & i_immediate)
/// - "srli": dest = src >> (0x1f & i_immediate) (logical)
/// - "srai": dest = src >> (0x1f & i_immediate) (arithmetic)
///
/// In RV64I, the shift operators
///
pub enum RegImm {
    Addi,
    Slti,
    Sltiu,
    Xori,
    Ori,
    Andi,
    Slli,
    Srli,
    Srai,
}

/// In RV32I and RV64I, perform an operation between the values in
/// src1 and src2 and place the result in dest
///
/// In RV32I, the operation performed is determined by the mnemonic
/// as follows:
/// - "add": dest = src1 + src2
/// - "sub": dest = src1 - src2
/// - "slt": dest = (src1 < src2) ? 1 : 0, signed comparison
/// - "sltu": dest = (src1 < src2) ? 1 : 0, unsigned comparison
/// - "and": dest = src1 & src2
/// - "or": dest = src1 | src2
/// - "xor": dest = src1 ^ src2
/// - "sll": dest = src1 << (0x1f & src2)
/// - "srl": dest = src1 >> (0x1f & src2) (logical)
/// - "sra": dest = src1 >> (0x1f & src2) (arithmetic)
///
/// In RV64I, the shift operators using the bottom 6 bits of
/// src2 as the shift amount: (0x3f & src2). In addition, the
/// following instructions operate on the low 32 bits of the
/// registers:
/// - "addw"
/// - "subw"
/// - "sllw"
/// - "srlw"
/// - "sraw"
///
pub enum RegReg {
    Add,
    Sub,
    Sll,
    Sltu,
    Xor,
    Srl,
    Sra,
    Or,
    And,
}

/*
/// RISC-V Instructions
///
/// Field names below correspond to the names in the
/// instruction set reference.
#[derive(Debug, Clone)]
pub enum Rv32i {
    /// In RV32I and RV64I, load u_immediate into dest[31:12] bits of
    /// dest, filling the low 12 bits with zeros. In RV64I, also sign
    /// extend the result to the high bits of dest. u_immediate is 20
    /// bits long.
    Lui(Utype),
    /// In RV32I, concatenate u_immediate with 12 low-order zeros, add
    /// pc to the the result, and place the result in dest. In RV64I,
    /// sign extend the result before adding to the pc. u_immediate is
    /// 20 bits long.
    Auipc(Utype),
    /// In RV32I and RV64I, store pc+4 in dest, and set pc = pc +
    /// offset, where offset is a multiple of 2. Offset is 21 bits
    /// long. An instruction-address-misaligned exception is generated
    /// if the target pc is not 4-byte aligned.
    Jal(Jtype),
    /// In RV32I and RV64I, store pc+4 in dest, compute base + offset,
    /// set bit 0 to zero, and set pc = result. The offset is 12
    /// bits long (and may be even or odd). An
    /// instruction-address-misaligned exception is generated if the
    /// target pc is not 4-byte aligned.
    Jalr(Itype),
}
*/

fn opcode_determined(opcode: u32, exec32: ExecFn32) -> Decoder {
    let next_mask = mask!(7); // opcode mask
    let mut value_map = HashMap::new();

    let executer = Decoder::Executer {
        xlen32_fn: Some(exec32),
    };
    value_map.insert(opcode, executer);
    Decoder::Decoder {
        next_mask,
        value_map,
    }
}

fn make_rv32i(&mut decoder: Decoder){

}


/*
pub fn make_rv32i() -> Vec<SignatureDecoder> {
    let mut vec = Vec::new();
    vec.push(opcode_determined(OP_LUI, ExecFn32(execute_lui_rv32i)));
    vec.push(opcode_determined(OP_AUIPC, ExecFn32(execute_auipc_rv32i)));
    vec.push(opcode_determined(OP_JAL, ExecFn32(execute_jal_rv32i)));
    vec.push(opcode_determined(OP_JALR, ExecFn32(execute_jalr_rv32i)));
    vec
}
*/
/// The purpose of this function is to combine the decoders for each
/// separate function into one decoder tree that will decode any of
/// the instructions covered by the inputs
pub fn combine_decoders(decoders: Vec<SignatureDecoder>) -> SignatureDecoder {

    // The decoders list (the argument) all have a next_mask.
    // Collect together those which have the same next mask. Store
    // them in this map for now. It maps next_mask values to vectors
    // of value_maps which have this next_mask
    let mut next_mask_to_value_maps = HashMap::new();
    
    for decoder in decoders {
	// If the decoder has a next_mask, and is not an executer, add
	// it to the map.
	// Note: at the top level, we are not expecting an executers. This
	// may indicate a problem with the structure of the program
	if let SignatureDecoder::Decoder { next_mask, value_map } = decoder {

	    match next_mask_to_value_maps.entry(next_mask) {
		Entry::Vacant(e) => { e.insert(vec![value_map]); },
		Entry::Occupied(mut e) => { e.get_mut().push(value_map); }
            }
	}
    }

    let mut decoders_vector
    
    // Now we have grouped together by mask, it is time to group together
    // by value. This will produce lists of decoders that need to be combined
    // again
    for (next_mask, value_maps_vector) in next_mask_to_value_maps.into_iter() {

	// value_maps_vector is a list of maps mapping values to decoders. This
	// needs to be converted into a map from values to vectors of decoders
	// (which correspond to the same value)
	let mut value_to_decoders_vector = HashMap::new();
	for value_map in value_maps_vector.into_iter() {
	    for (value, decoder) in value_map.into_iter() {
		match value_to_decoders_vector.entry(value) {
		    Entry::Vacant(e) => { e.insert(vec![decoder]); },
		    Entry::Occupied(mut e) => { e.get_mut().push(decoder); }
		}
		
	    }
	}

	// Now that values are mapped to decoders, each of these decoder vectors
	// can be combined into one using this function (recursively)
	let mut value_map = HashMap::new();
	for (value, decoders) in value_to_decoders_vector.into_iter() {
	    value_map.insert(value, combine_decoders(decoders));
	}

	// Create this decoder and add it to the back
	let 
	
    }

    SignatureDecoder::Decoder { next_mask: 0, value_map: HashMap::new() }
}
*/
