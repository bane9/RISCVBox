use crate::backend::{
    common, core::amd64_reg, core::emit_mov_reg_guest_to_host, core::emit_mov_reg_host_to_guest,
};
use crate::cpu::CpuReg;
use crate::*;
use common::{DecodeRet, HostEncodedInsn};

pub struct RvmImpl;

impl common::Rvm for RvmImpl {
    fn emit_mul(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        emit_imul32_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_mulh(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);

        emit_imul_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, 32);

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_mulhsu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);

        emit_imul_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RBX);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, 32);

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_mulhu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        emit_mul_reg!(insn, amd64_reg::RBX);

        emit_shr_reg_imm!(insn, amd64_reg::RAX, 32);

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_div(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let mut set_constant_insn = HostEncodedInsn::new();
        emit_mov_reg_imm_auto!(set_constant_insn, amd64_reg::RAX, CpuReg::MAX);

        let mut div_insn = HostEncodedInsn::new();
        emit_movsxd_reg_reg!(div_insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(div_insn, amd64_reg::RBX, amd64_reg::RBX);
        emit_cqo!(div_insn);
        emit_idiv_reg!(div_insn, amd64_reg::RBX);
        emit_jmp_imm32!(div_insn, set_constant_insn.size());

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        // if (RBX != 0)
        emit_cmp_reg_imm!(insn, amd64_reg::RBX, 0);
        emit_jz_imm!(insn, div_insn.size());
        // {
        insn.push_slice(div_insn.as_slice());
        // } else {
        insn.push_slice(set_constant_insn.as_slice());
        // }

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_divu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let mut set_constant_insn = HostEncodedInsn::new();
        emit_mov_reg_imm_auto!(set_constant_insn, amd64_reg::RAX, CpuReg::MAX);

        let mut div_insn = HostEncodedInsn::new();
        emit_div_reg!(div_insn, amd64_reg::RBX);
        emit_jmp_imm32!(div_insn, set_constant_insn.size());

        emit_xor_reg_reg!(insn, amd64_reg::RDX, amd64_reg::RDX);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        // if (RBX != 0)
        emit_cmp_reg_imm!(insn, amd64_reg::RBX, 0);
        emit_jz_imm!(insn, div_insn.size());
        // {
        insn.push_slice(div_insn.as_slice());
        // } else {
        insn.push_slice(set_constant_insn.as_slice());
        // }

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_rem(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let mut div_insn = HostEncodedInsn::new();
        emit_movsxd_reg_reg!(div_insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(div_insn, amd64_reg::RBX, amd64_reg::RBX);
        emit_cqo!(div_insn);
        emit_div_reg!(div_insn, amd64_reg::RBX);
        emit_mov_reg_reg1!(div_insn, amd64_reg::RAX, amd64_reg::RDX);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        // if (RBX != 0)
        emit_cmp_reg_imm!(insn, amd64_reg::RBX, 0);
        emit_jz_imm!(insn, div_insn.size());
        // {
        insn.push_slice(div_insn.as_slice());
        // }

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_remu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let mut div_insn = HostEncodedInsn::new();
        emit_idiv_reg!(div_insn, amd64_reg::RBX);
        emit_mov_reg_reg1!(div_insn, amd64_reg::RAX, amd64_reg::RDX);

        emit_xor_reg_reg!(insn, amd64_reg::RDX, amd64_reg::RDX);

        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host(&mut insn, cpu, amd64_reg::RBX, rs2);

        // if (RBX != 0)
        emit_cmp_reg_imm!(insn, amd64_reg::RBX, 0);
        emit_jz_imm!(insn, div_insn.size());
        // {
        insn.push_slice(div_insn.as_slice());
        // }

        emit_mov_reg_host_to_guest(&mut insn, cpu, amd64_reg::RBX, amd64_reg::RAX, rd);

        Ok(insn)
    }
}
