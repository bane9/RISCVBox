use crate::backend::*;
use crate::cpu::*;

pub fn decode_privledged(cpu: &mut Cpu, ptr: *mut u8, insn: u32) -> DecodeRet {
    Err(JitError::InvalidInstruction(insn))
}
