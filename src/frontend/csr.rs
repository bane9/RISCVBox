use crate::{backend::*, cpu::OpType};

pub fn decode_csr(insn: u32) -> DecodeRet {
    let opcode = insn & 0x7f;

    let result: DecodeRet = match OpType::from_u32(opcode) {
        OpType::CSR => {
            let funct3 = (insn & 0x03000000) >> 25;
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let csr = ((insn >> 20) & 0xfff) as u16;

            match funct3 {
                0b001 => CsrImpl::emit_csrrw(rd, rs1, csr),
                0b010 => CsrImpl::emit_csrrs(rd, rs1, csr),
                0b011 => CsrImpl::emit_csrrc(rd, rs1, csr),
                0b101 => CsrImpl::emit_csrrwi(rd, rs1, csr),
                0b110 => CsrImpl::emit_csrrsi(rd, rs1, csr),
                0b111 => CsrImpl::emit_csrrci(rd, rs1, csr),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        _ => Err(JitError::InvalidInstruction(insn)),
    };

    result
}
