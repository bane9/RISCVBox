use crate::cpu::*;
use crate::backend::*;

pub fn decode_rvi(cpu: &mut Cpu, ptr: *mut u8, insn: u32) -> Result<(), JitError> {
    let opcode = insn & 0x7f;

    let result: Result<(), JitError> = match OpType::from_u32(opcode) {
        OpType::I => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;

            match funct3 {
                0b000 => RviImpl::emit_addi(ptr, cpu, rd, rs1, imm),
                0b001 => RviImpl::emit_slli(ptr, cpu, rd, rs1, (imm & 0b11111) as u8),
                0b010 => RviImpl::emit_slti(ptr, cpu, rd, rs1, imm),
                0b011 => RviImpl::emit_sltiu(ptr, cpu, rd, rs1, imm),
                0b100 => RviImpl::emit_xori(ptr, cpu, rd, rs1, imm),
                0b110 => RviImpl::emit_ori(ptr, cpu, rd, rs1, imm),
                0b111 => RviImpl::emit_andi(ptr, cpu, rd, rs1, imm),
                0b101 => {
                    let funct7 = ((insn >> 25) & 0b1111111) as u8;
                    match funct7 {
                        0b0000000 => RviImpl::emit_srli(ptr, cpu, rd, rs1, (imm & 0b11111) as u8),
                        0b0100000 => RviImpl::emit_srai(ptr, cpu, rd, rs1, (imm & 0b11111) as u8),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                _ => Err(JitError::InvalidInstruction(insn))
            }
        },
        OpType::B => {
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let imm = ((insn >> 7) & 0b11111) as i32;

            match funct3 {
                0b000 => RviImpl::emit_beq(ptr, cpu, rs1, rs2, imm),
                0b001 => RviImpl::emit_bne(ptr, cpu, rs1, rs2, imm),
                0b100 => RviImpl::emit_blt(ptr, cpu, rs1, rs2, imm),
                0b101 => RviImpl::emit_bge(ptr, cpu, rs1, rs2, imm),
                0b110 => RviImpl::emit_bltu(ptr, cpu, rs1, rs2, imm),
                0b111 => RviImpl::emit_bgeu(ptr, cpu, rs1, rs2, imm),
                _ => Err(JitError::InvalidInstruction(insn))
            }
        },
        OpType::R => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let funct7 = ((insn >> 25) & 0b1111111) as u8;

            match funct3 {
                0b000 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_add(ptr, cpu, rd, rs1, rs2),
                        0b0100000 => RviImpl::emit_sub(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b001 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_sll(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b010 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_slt(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b011 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_sltu(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b100 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_xor(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b101 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_srl(ptr, cpu, rd, rs1, rs2),
                        0b0100000 => RviImpl::emit_sra(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b110 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_or(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                0b111 => {
                    match funct7 {
                        0b0000000 => RviImpl::emit_and(ptr, cpu, rd, rs1, rs2),
                        _ => Err(JitError::InvalidInstruction(insn))
                    }
                },
                _ => Err(JitError::InvalidInstruction(insn))
            }
        },
        OpType::U => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let imm = ((insn >> 12) & 0b11111111111111111111) as i32;

            match ((insn >> 25) & 0b1111111) as u8 {
                0b0010111 => RviImpl::emit_auipc(ptr, cpu, rd, imm),
                0b0110111 => RviImpl::emit_lui(ptr, cpu, rd, imm),
                _ => Err(JitError::InvalidInstruction(insn))
            }
        },
        OpType::JAL => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let imm = ((insn >> 12) & 0b11111111111111111111) as i32;

            RviImpl::emit_jal(ptr, cpu, rd, imm)
        },
        OpType::JALR => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;

            RviImpl::emit_jalr(ptr, cpu, rd, rs1, imm)
        },
        OpType::L => {
            let rd = ((insn >> 7) & 0b11111) as u8;
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let imm = ((insn >> 20) & 0b111111111111) as i32;

            match funct3 {
                0b000 => RviImpl::emit_lb(ptr, cpu, rd, rs1, imm),
                0b001 => RviImpl::emit_lh(ptr, cpu, rd, rs1, imm),
                0b010 => RviImpl::emit_lw(ptr, cpu, rd, rs1, imm),
                0b100 => RviImpl::emit_lbu(ptr, cpu, rd, rs1, imm),
                0b101 => RviImpl::emit_lhu(ptr, cpu, rd, rs1, imm),
                _ => Err(JitError::InvalidInstruction(insn))
            }
        },
        OpType::S => {
            let funct3 = ((insn >> 12) & 0b111) as u8;
            let rs1 = ((insn >> 15) & 0b11111) as u8;
            let rs2 = ((insn >> 20) & 0b11111) as u8;
            let imm = (((insn >> 7) & 0b11111) | ((insn >> 25) & 0b1111111) << 5) as i32;

            match funct3 {
                0b000 => RviImpl::emit_sb(ptr, cpu, rs1, rs2, imm),
                0b001 => RviImpl::emit_sh(ptr, cpu, rs1, rs2, imm),
                0b010 => RviImpl::emit_sw(ptr, cpu, rs1, rs2, imm),
                _ => Err(JitError::InvalidInstruction(insn))
            }
        },
        OpType::FENCE => {
            let pred = ((insn >> 24) & 0b1111) as u8;
            let succ = ((insn >> 20) & 0b1111) as u8;

            RviImpl::emit_fence(ptr, cpu, pred, succ)
        },
        _ => Err(JitError::InvalidInstruction(insn))
    };

    result
}
