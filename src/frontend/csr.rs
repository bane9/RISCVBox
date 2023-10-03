use crate::backend::*;


pub fn decode_csr(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
