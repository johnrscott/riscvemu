use crate::{
    decode::{Decoder, DecoderError, MaskWithValue},
    opcodes::OP_LUI,
    utils::mask,
};

use super::{eei::Eei, rv32i::execute_lui, ExecuteInstr};

/// The intention of this kind of function (generic on EEI) is to provide
/// a way to separate the decoding of the instruction from the actual
/// implementation of the execution environment
pub fn opcode_determined<E: Eei>(
    decoder: &mut Decoder<ExecuteInstr<E>>,
    opcode: u32,
    exec: ExecuteInstr<E>,
) -> Result<(), DecoderError> {
    let masks_with_values = vec![MaskWithValue {
        mask: mask(7),
        value: opcode,
    }];
    decoder.push_instruction(masks_with_values, exec)
}

pub fn make_rv32i<E: Eei>(decoder: &mut Decoder<ExecuteInstr<E>>) -> Result<(), DecoderError> {
    // Opcode determines instruction
    opcode_determined(decoder, OP_LUI, execute_lui)
}
