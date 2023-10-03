use crate::backend::*;


pub fn decode_rvm(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
