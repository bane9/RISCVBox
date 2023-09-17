use crate::backend::common;
use common::DecodeRet;

pub struct RviImpl;

impl common::Rvi for RviImpl {
    fn emit_addi(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_add(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sub(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slli(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slti(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sltiu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_xori(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_srli(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_srai(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_ori(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_andi(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lui(cpu: &mut crate::cpu::Cpu, rd: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_auipc(cpu: &mut crate::cpu::Cpu, rd: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_jal(cpu: &mut crate::cpu::Cpu, rd: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_jalr(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_beq(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bne(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_blt(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bge(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bltu(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bgeu(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lb(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lh(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lw(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lbu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lhu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sb(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sh(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sw(cpu: &mut crate::cpu::Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_fence(cpu: &mut crate::cpu::Cpu, pred: u8, succ: u8) -> DecodeRet {
        todo!()
    }

    fn emit_fence_i(cpu: &mut crate::cpu::Cpu) -> DecodeRet {
        todo!()
    }

    fn emit_ecall(cpu: &mut crate::cpu::Cpu) -> DecodeRet {
        todo!()
    }

    fn emit_ebreak(cpu: &mut crate::cpu::Cpu) -> DecodeRet {
        todo!()
    }

    fn emit_xor(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_srl(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sra(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_or(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_and(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sll(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slt(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sltu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }
}
