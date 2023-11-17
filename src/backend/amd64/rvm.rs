use crate::backend::{common, core::amd64_reg};
use crate::*;
use common::{DecodeRet, HostEncodedInsn};

pub struct RvmImpl;

impl common::Rvm for RvmImpl {
    fn emit_mul(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);

        emit_imul_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_mulh(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);

        emit_imul_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, 32);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_mulhsu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);

        emit_imul_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, 32);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_mulhu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);

        emit_mul_reg!(insn, amd64_reg::RBX);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, 32);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_div(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);

        // TODO: check if rbx is 0

        emit_idiv_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_divu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);

        // TODO: check if rbx is 0

        emit_div_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_rem(rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        // let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        Ok(insn)
    }

    fn emit_remu(rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        // let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        Ok(insn)
    }
}
