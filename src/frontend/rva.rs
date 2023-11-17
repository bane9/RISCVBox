use crate::backend::*;

pub fn decode_rva(insn: u32) -> DecodeRet {
    let opcode = insn & 0x7f;

    if opcode != 0x2f {
        return Err(JitError::InvalidInstruction(insn));
    }

    let funct7 = (insn >> 25) & 0x7f;
    let funct5 = (funct7 & 0x7c) >> 2;

    let rd = ((insn >> 7) & 0x1f) as u8;
    let rs1 = ((insn >> 15) & 0x1f) as u8;
    let rs2 = ((insn >> 20) & 0x1f) as u8;

    let aq = ((insn >> 26) & 0x1) == 1;
    let rl = ((insn >> 25) & 0x1) == 1;

    match funct5 {
        0x00 => RvaImpl::emit_amoadd_w(rd, rs1, rs2, aq, rl),
        0x01 => RvaImpl::emit_amoswap_w(rd, rs1, rs2, aq, rl),
        0x02 => RvaImpl::emit_lr_w(rd, rs1, aq, rl),
        0x03 => RvaImpl::emit_sc_w(rd, rs1, rs2, aq, rl),
        0x04 => RvaImpl::emit_amoxor_w(rd, rs1, rs2, aq, rl),
        0x08 => RvaImpl::emit_amoor_w(rd, rs1, rs2, aq, rl),
        0x0c => RvaImpl::emit_amoand_w(rd, rs1, rs2, aq, rl),
        0x10 => RvaImpl::emit_amomin_w(rd, rs1, rs2, aq, rl),
        0x14 => RvaImpl::emit_amomax_w(rd, rs1, rs2, aq, rl),
        0x18 => RvaImpl::emit_amominu_w(rd, rs1, rs2, aq, rl),
        0x1c => RvaImpl::emit_amomaxu_w(rd, rs1, rs2, aq, rl),
        _ => Err(JitError::InvalidInstruction(insn)),
    }
}
