use crate::backend::*;
use crate::cpu::*;

pub fn decode_rvi(cpu: &mut Cpu, ptr: *mut u8, insn: u32) -> DecodeRet {
    let opcode = insn & 0x7f;

    let result: DecodeRet = match OpType::from_u32(opcode) {
        OpType::I => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;

            match funct3 {
                0b000 => RviImpl::emit_addi(cpu, rd, rs1, imm),
                0b001 => RviImpl::emit_slli(cpu, rd, rs1, (imm & 0b11111) as u8),
                0b010 => RviImpl::emit_slti(cpu, rd, rs1, imm),
                0b011 => RviImpl::emit_sltiu(cpu, rd, rs1, imm),
                0b100 => RviImpl::emit_xori(cpu, rd, rs1, imm),
                0b110 => RviImpl::emit_ori(cpu, rd, rs1, imm),
                0b111 => RviImpl::emit_andi(cpu, rd, rs1, imm),
                0b101 => {
                    let funct7 = ((insn >> 25) & 0b1111111) as u8;
                    match funct7 {
                        0b0000000 => RviImpl::emit_srli(cpu, rd, rs1, (imm & 0b11111) as u8),
                        0b0100000 => RviImpl::emit_srai(cpu, rd, rs1, (imm & 0b11111) as u8),
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
            let imm = ((insn >> 7) & 0b11111) as i32;

            match funct3 {
                0b000 => RviImpl::emit_beq(cpu, rs1, rs2, imm),
                0b001 => RviImpl::emit_bne(cpu, rs1, rs2, imm),
                0b100 => RviImpl::emit_blt(cpu, rs1, rs2, imm),
                0b101 => RviImpl::emit_bge(cpu, rs1, rs2, imm),
                0b110 => RviImpl::emit_bltu(cpu, rs1, rs2, imm),
                0b111 => RviImpl::emit_bgeu(cpu, rs1, rs2, imm),
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
                    0b0000000 => RviImpl::emit_add(cpu, rd, rs1, rs2),
                    0b0100000 => RviImpl::emit_sub(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b001 => match funct7 {
                    0b0000000 => RviImpl::emit_sll(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b010 => match funct7 {
                    0b0000000 => RviImpl::emit_slt(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b011 => match funct7 {
                    0b0000000 => RviImpl::emit_sltu(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b100 => match funct7 {
                    0b0000000 => RviImpl::emit_xor(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b101 => match funct7 {
                    0b0000000 => RviImpl::emit_srl(cpu, rd, rs1, rs2),
                    0b0100000 => RviImpl::emit_sra(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b110 => match funct7 {
                    0b0000000 => RviImpl::emit_or(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                0b111 => match funct7 {
                    0b0000000 => RviImpl::emit_and(cpu, rd, rs1, rs2),
                    _ => Err(JitError::InvalidInstruction(insn)),
                },
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::U => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let imm = ((insn >> 12) & 0b11111111111111111111) as i32;

            match ((insn >> 25) & 0b1111111) as u8 {
                0b0010111 => RviImpl::emit_auipc(cpu, rd, imm),
                0b0110111 => RviImpl::emit_lui(cpu, rd, imm),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::JAL => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let imm = ((insn >> 12) & 0b11111111111111111111) as i32;

            RviImpl::emit_jal(cpu, rd, imm)
        }
        OpType::JALR => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;

            RviImpl::emit_jalr(cpu, rd, rs1, imm)
        }
        OpType::L => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;

            match funct3 {
                0b000 => RviImpl::emit_lb(cpu, rd, rs1, imm),
                0b001 => RviImpl::emit_lh(cpu, rd, rs1, imm),
                0b010 => RviImpl::emit_lw(cpu, rd, rs1, imm),
                0b100 => RviImpl::emit_lbu(cpu, rd, rs1, imm),
                0b101 => RviImpl::emit_lhu(cpu, rd, rs1, imm),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::S => {
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let imm = (((insn >> 7) & 0b11111) | ((insn >> 25) & 0b1111111) << 5) as i32;

            match funct3 {
                0b000 => RviImpl::emit_sb(cpu, rs1, rs2, imm),
                0b001 => RviImpl::emit_sh(cpu, rs1, rs2, imm),
                0b010 => RviImpl::emit_sw(cpu, rs1, rs2, imm),
                _ => Err(JitError::InvalidInstruction(insn)),
            }
        }
        OpType::FENCE => {
            let pred = ((insn >> 24) & 0b1111) as u8;
            let succ = ((insn >> 20) & 0b1111) as u8;

            RviImpl::emit_fence(cpu, pred, succ)
        }
        _ => Err(JitError::InvalidInstruction(insn)),
    };

    result
}
