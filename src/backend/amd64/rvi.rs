use crate::backend::common;
use common::PtrT;

pub struct RviImpl;

impl common::Rvi for RviImpl {
    fn emit_addi(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_add(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sub(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_slli(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        shamt: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_slti(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sltiu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_xori(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_srli(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        shamt: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_srai(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        shamt: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_ori(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_andi(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_lui(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_auipc(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_jal(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_jalr(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_beq(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_bne(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_blt(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_bge(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_bltu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_bgeu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_lb(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_lh(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_lw(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_lbu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_lhu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sb(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sh(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sw(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rs1: u8,
        rs2: u8,
        imm: i32,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_fence(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        pred: u8,
        succ: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_fence_i(ptr: PtrT, cpu: &mut crate::cpu::Cpu) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_ecall(ptr: PtrT, cpu: &mut crate::cpu::Cpu) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_ebreak(ptr: PtrT, cpu: &mut crate::cpu::Cpu) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_xor(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_srl(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sra(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_or(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_and(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sll(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_slt(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_sltu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }
}
