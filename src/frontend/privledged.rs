use crate::backend::*;


pub fn decode_privledged(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
