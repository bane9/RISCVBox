use crate::backend::common;
use crate::backend::target::core::{amd64_reg, BackendCore, BackendCoreImpl};
use crate::cpu::csr;
use crate::cpu::{self, Exception};
use crate::*;
use common::{
    BusAccessVars, DecodeRet, HostEncodedInsn, JumpCond, JumpVars, PCAccess, UsizeConversions,
};

pub struct RviImpl;

fn emit_jmp(mut cond: JumpVars) -> HostEncodedInsn {
    cond.set_pc(cpu::get_cpu().pc);

    let mut insn =
        BackendCoreImpl::emit_usize_call_with_1_arg(common::c_jump_resolver_cb, cond.to_usize());

    emit_cmp_reg_imm!(insn, amd64_reg::RAX, 0);

    let mut jmp_insn = HostEncodedInsn::new();
    emit_jmp_reg!(jmp_insn, amd64_reg::RAX);

    emit_jz_imm!(insn, jmp_insn.size() as u8);
    insn.push_slice(jmp_insn.iter().as_slice());

    insn
}

macro_rules! emit_bus_access {
    ($cond: expr) => {{
        $cond.set_pc(cpu::get_cpu().pc);

        BackendCoreImpl::emit_void_call_with_1_arg(common::c_bus_resolver_cb, $cond.to_usize())
    }};
}

impl common::Rvi for RviImpl {
    fn emit_addi(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);

        emit_add_reg_imm!(insn, amd64_reg::RBX, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_add(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_add_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_sub(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_sub_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_slli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);

        emit_shl_reg_imm!(insn, amd64_reg::RBX, shamt as u8);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_slti(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);

        emit_test_less_reg_imm!(insn, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_sltiu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);

        emit_test_less_reg_imm!(insn, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_xori(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);

        emit_test_less_reg_imm!(insn, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_srli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, shamt);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_srai(rd: u8, rs1: u8, shamt: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);

        emit_shr_reg_imm!(insn, amd64_reg::RBX, shamt);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_ori(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);

        emit_or_reg_imm!(insn, amd64_reg::RBX, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_andi(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);

        emit_and_reg_imm!(insn, amd64_reg::RBX, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_lui(rd: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let rd_addr = &cpu.regs[rd as usize] as *const _ as usize;

        emit_move_reg_imm!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_imm!(insn, amd64_reg::RBX, imm << 12 as u32);

        Ok(insn)
    }

    fn emit_auipc(rd: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        let rd_addr = &cpu.regs[rd as usize] as *const _ as usize;

        emit_move_reg_imm!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_imm!(insn, amd64_reg::RBX, cpu.pc + imm as u32);

        Ok(insn)
    }

    fn emit_jal(rd: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::Always,
            imm,
            rd as u32,
            0x0u32,
        )))
    }

    fn emit_jalr(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::AlwaysAbsolute,
            imm,
            rd as u32,
            rs1 as u32,
        )))
    }

    fn emit_beq(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::Equal,
            imm,
            rs1 as u32,
            rs2 as u32,
        )))
    }

    fn emit_bne(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::NotEqual,
            imm,
            rs1 as u32,
            rs2 as u32,
        )))
    }

    fn emit_blt(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::LessThan,
            imm,
            rs1 as u32,
            rs2 as u32,
        )))
    }

    fn emit_bge(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::GreaterThanEqual,
            imm,
            rs1 as u32,
            rs2 as u32,
        )))
    }

    fn emit_bltu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::LessThanUnsigned,
            imm,
            rs1 as u32,
            rs2 as u32,
        )))
    }

    fn emit_bgeu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpVars::new(
            JumpCond::GreaterThanEqualUnsigned,
            imm,
            rs1 as u32,
            rs2 as u32,
        )))
    }

    fn emit_lb(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::LoadByte,
            imm,
            rd as u32,
            rs1 as u32,
        )))
    }

    fn emit_lh(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::LoadHalf,
            imm,
            rd as u32,
            rs1 as u32,
        )))
    }

    fn emit_lw(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::LoadWord,
            imm,
            rd as u32,
            rs1 as u32,
        )))
    }

    fn emit_lbu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::LoadByteUnsigned,
            imm,
            rd as u32,
            rs1 as u32,
        )))
    }

    fn emit_lhu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::LoadHalfUnsigned,
            imm,
            rd as u32,
            rs1 as u32,
        )))
    }

    fn emit_sb(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::StoreByte,
            imm,
            rs2 as u32, // Intentionally flipped
            rs1 as u32,
        )))
    }

    fn emit_sh(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::StoreHalf,
            imm,
            rs2 as u32,
            rs1 as u32,
        )))
    }

    fn emit_sw(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_bus_access!(BusAccessVars::new(
            backend::BusAccessCond::StoreWord,
            imm,
            rs2 as u32,
            rs1 as u32,
        )))
    }

    fn emit_fence(_pred: u8, _succ: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();

        emit_nop!(insn);

        Ok(insn)
    }

    fn emit_fence_i() -> DecodeRet {
        let mut insn = HostEncodedInsn::new();

        emit_nop!(insn);

        Ok(insn)
    }

    fn emit_ecall() -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        match cpu.csr.read_mpp_mode() {
            csr::MppMode::User => {
                emit_set_exception!(insn, cpu, Exception::EnvironmentCallFromUMode);
            }
            csr::MppMode::Supervisor => {
                emit_set_exception!(insn, cpu, Exception::EnvironmentCallFromSMode);
            }
            csr::MppMode::Machine => {
                emit_set_exception!(insn, cpu, Exception::EnvironmentCallFromMMode);
            }
        }

        Ok(insn)
    }

    fn emit_ebreak() -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_set_exception!(insn, cpu, Exception::Breakpoint);

        Ok(insn)
    }

    fn emit_xor(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_xor_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_srl(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_shr_reg_cl!(insn, amd64_reg::RBX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_sra(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_test_greater_reg_imm!(insn, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_or(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_or_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_and(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_and_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_sll(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_shl_reg_cl!(insn, amd64_reg::RBX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_slt(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_test_less_reg_imm!(insn, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_sltu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_test_less_reg_imm!(insn, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }
}
