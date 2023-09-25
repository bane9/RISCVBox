use crate::backend::common;
use common::DecodeRet;

pub struct RviImpl;

impl common::Rvi for RviImpl {
    fn emit_addi(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_add(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sub(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slti(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sltiu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_xori(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_srli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_srai(rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_ori(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_andi(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lui(rd: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_auipc(rd: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_jal(rd: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_jalr(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_beq(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bne(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_blt(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bge(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bltu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bgeu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lb(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lh(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lw(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lbu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lhu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sb(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sh(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sw(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_fence(pred: u8, succ: u8) -> DecodeRet {
        todo!()
    }

    fn emit_fence_i() -> DecodeRet {
        todo!()
    }

    fn emit_ecall() -> DecodeRet {
        todo!()
    }

    fn emit_ebreak() -> DecodeRet {
        todo!()
    }

    fn emit_xor(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_srl(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sra(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_or(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_and(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sll(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slt(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sltu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }
}
