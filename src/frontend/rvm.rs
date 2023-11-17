use crate::{backend::*, cpu::OpType};

pub fn decode_rvm(insn: u32) -> DecodeRet {
    // only div, divu, rem, remu, mul, mulh, mulhsu, mulhu are implemented

    let opcode = insn & 0x7f;

    let result = match OpType::from_u32(opcode) {
        OpType::R => {
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let funct7 = ((insn >> 25) & 0b1111111) as u8;
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;

            match funct3 {
                0b000 => match funct7 {
                    0b0000001 => RvmImpl::emit_mul(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b001 => match funct7 {
                    0b0000001 => RvmImpl::emit_mulh(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b010 => match funct7 {
                    0b0000001 => RvmImpl::emit_mulhsu(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b011 => match funct7 {
                    0b0000001 => RvmImpl::emit_mulhu(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b100 => match funct7 {
                    0b0000001 => RvmImpl::emit_div(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b101 => match funct7 {
                    0b0000001 => RvmImpl::emit_divu(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b110 => match funct7 {
                    0b0000001 => RvmImpl::emit_rem(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b111 => match funct7 {
                    0b0000001 => RvmImpl::emit_remu(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        _ => Err(JitError::InvalidInstruction(insn)),
    };

    result
}
