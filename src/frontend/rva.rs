use crate::backend::*;
use crate::cpu::*;

pub fn decode_rva(insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
