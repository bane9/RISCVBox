use crate::backend::*;
use crate::cpu::*;

pub fn decode_csr(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
