use crate::backend::*;


pub fn decode_rva(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
