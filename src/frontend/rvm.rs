use crate::backend::*;
use crate::cpu::*;

pub fn decode_rvm(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
