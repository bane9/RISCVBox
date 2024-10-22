use crate::backend::*;
use crate::cpu::*;
use crate::util::sign_extend;

pub fn decode_rvi(insn: u32) -> DecodeRet {
    let opcode = insn & 0x7f;

    let result: DecodeRet = match OpType::from_u32(opcode) {
        OpType::I => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;
            let imm = sign_extend(imm, 12) as i32;

            match funct3 {
                0b000 => RviImpl::emit_addi(rd, rs1, imm),
                0b001 => {
                    let imm = imm & 0b111111;

                    if imm & (1 << 5) != 0 {
                        return Err(JitError::InvalidInstruction(insn));
                    }

                    RviImpl::emit_slli(rd, rs1, (imm & 0b11111) as u8)
                }
                0b010 => RviImpl::emit_slti(rd, rs1, imm),
                0b011 => RviImpl::emit_sltiu(rd, rs1, imm),
                0b100 => RviImpl::emit_xori(rd, rs1, imm),
                0b110 => RviImpl::emit_ori(rd, rs1, imm),
                0b111 => RviImpl::emit_andi(rd, rs1, imm),
                0b101 => {
                    let funct7 = ((insn >> 25) & 0b1111111) as u8;
                    let imm = (insn >> 20) & 0x1f;
                    match funct7 {
                        0b0000000 => RviImpl::emit_srli(rd, rs1, imm as u8),
                        0b0100000 => RviImpl::emit_srai(rd, rs1, imm as u8),
                        _ => Err(JitError::InvalidInstruction(insn)),
                    }
                }
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::B => {
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let imm = ((insn & 0xf00) >> 7)
                | ((insn & 0x7e000000) >> 20)
                | ((insn & 0x80) << 4)
                | ((insn >> 31) << 12);

            let imm = if imm & 0x1000 != 0 {
                imm | 0xffffe000
            } else {
                imm
            };

            let imm = imm as i32;

            match funct3 {
                0b000 => RviImpl::emit_beq(rs1, rs2, imm),
                0b001 => RviImpl::emit_bne(rs1, rs2, imm),
                0b100 => RviImpl::emit_blt(rs1, rs2, imm),
                0b101 => RviImpl::emit_bge(rs1, rs2, imm),
                0b110 => RviImpl::emit_bltu(rs1, rs2, imm),
                0b111 => RviImpl::emit_bgeu(rs1, rs2, imm),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::R => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let funct7 = ((insn >> 25) & 0b1111111) as u8;

            match funct3 {
                0b000 => match funct7 {
                    0b0000000 => RviImpl::emit_add(rd, rs1, rs2),
                    0b0100000 => RviImpl::emit_sub(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b001 => match funct7 {
                    0b0000000 => RviImpl::emit_sll(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b010 => match funct7 {
                    0b0000000 => RviImpl::emit_slt(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b011 => match funct7 {
                    0b0000000 => RviImpl::emit_sltu(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b100 => match funct7 {
                    0b0000000 => RviImpl::emit_xor(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b101 => match funct7 {
                    0b0000000 => RviImpl::emit_srl(rd, rs1, rs2),
                    0b0100000 => RviImpl::emit_sra(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b110 => match funct7 {
                    0b0000000 => RviImpl::emit_or(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b111 => match funct7 {
                    0b0000000 => RviImpl::emit_and(rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::AUIPC => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let imm = (insn & 0xfffff000) as i32;

            RviImpl::emit_auipc(rd, imm)
        }
        OpType::U => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let imm = (insn & 0xfffff000) as i32;

            RviImpl::emit_lui(rd, imm)
        }
        OpType::JAL => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let mut imm: u32 = ((insn & 0x80000000) >> 11)
                | ((insn & 0x7fe00000) >> 20)
                | ((insn & 0x00100000) >> 9)
                | (insn & 0x000ff000);

            if (imm & 0x00100000) != 0 {
                imm |= 0xffe00000;
            }

            RviImpl::emit_jal(rd, imm as i32)
        }
        OpType::JALR => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0xfff) as i32;
            let imm = sign_extend(imm, 12) as i32;

            RviImpl::emit_jalr(rd, rs1, imm)
        }
        OpType::L => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = (insn >> 20) as i32;
            let imm = sign_extend(imm, 12) as i32;

            match funct3 {
                0b000 => RviImpl::emit_lb(rd, rs1, imm),
                0b001 => RviImpl::emit_lh(rd, rs1, imm),
                0b010 => RviImpl::emit_lw(rd, rs1, imm),
                0b100 => RviImpl::emit_lbu(rd, rs1, imm),
                0b101 => RviImpl::emit_lhu(rd, rs1, imm),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::S => {
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let imm = (((insn >> 7) & 0x1f) | ((insn & 0xfe000000) >> 20)) as i32;
            let imm = sign_extend(imm, 12) as i32;

            match funct3 {
                0b000 => RviImpl::emit_sb(rs1, rs2, imm),
                0b001 => RviImpl::emit_sh(rs1, rs2, imm),
                0b010 => RviImpl::emit_sw(rs1, rs2, imm),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::FENCE => {
            let pred = ((insn >> 24) & 0b1111) as u8;
            let succ = ((insn >> 20) & 0b1111) as u8;

            RviImpl::emit_fence(pred, succ)
        }
        _ => Err(JitError::InvalidInstruction(insn)),
    };

    result
}
