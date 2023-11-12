use crate::backend::common;
use crate::backend::returnable::{ReturnableHandler, ReturnableImpl};
use crate::backend::target::core::{amd64_reg, BackendCore, BackendCoreImpl};
use crate::bus::bus::*;
use crate::cpu::{self, Exception, PrivMode};
use crate::*;
use common::{DecodeRet, HostEncodedInsn, JumpCond, JumpVars};

pub struct RviImpl;

macro_rules! emit_bus_access {
    ($addr_reg:expr, $data_reg:expr, $size:expr, $imm:expr, $write:expr, $signed:expr) => {{
        BackendCoreImpl::emit_usize_call_with_4_args(
            c_bus_access,
            $addr_reg as usize,
            $data_reg as usize,
            (($imm as usize) << 8
                | ($size as usize) << 2
                | (($write as usize) << 1)
                | $signed as usize),
            cpu::get_cpu().pc as usize,
        )
    }};
}

extern "C" fn jump_resolver_cb(jmp_cond: usize) -> usize {
    let cpu = cpu::get_cpu();
    let jmp_cond = JumpVars::from_usize(jmp_cond);

    let (jmp_addr, should_jmp) = match jmp_cond.cond {
        JumpCond::Always => {
            let pc = jmp_cond.pc as i64;
            let pc = pc.wrapping_add(jmp_cond.imm as i64);

            (pc as u32, true)
        }
        JumpCond::AlwaysAbsolute => {
            let pc = cpu.regs[jmp_cond.reg2 as usize] as i64;
            let pc = pc.wrapping_add(jmp_cond.imm as i64);

            (pc as u32, true)
        }
        JumpCond::Equal => {
            if cpu.regs[jmp_cond.reg1 as usize] as i32 == cpu.regs[jmp_cond.reg2 as usize] as i32 {
                let pc = jmp_cond.pc as i64;
                let pc = pc.wrapping_add(jmp_cond.imm as i64);

                (pc as u32, true)
            } else {
                (0, false)
            }
        }
        JumpCond::NotEqual => {
            if cpu.regs[jmp_cond.reg1 as usize] as i32 != cpu.regs[jmp_cond.reg2 as usize] as i32 {
                let pc = jmp_cond.pc as i64;
                let pc = pc.wrapping_add(jmp_cond.imm as i64);

                (pc as u32, true)
            } else {
                (0, false)
            }
        }
        JumpCond::LessThan => {
            if (cpu.regs[jmp_cond.reg1 as usize] as i32) < (cpu.regs[jmp_cond.reg2 as usize] as i32)
            {
                let pc = jmp_cond.pc as i64;
                let pc = pc.wrapping_add(jmp_cond.imm as i64);

                (pc as u32, true)
            } else {
                (0, false)
            }
        }
        JumpCond::GreaterThanEqual => {
            if (cpu.regs[jmp_cond.reg1 as usize] as i32) < (cpu.regs[jmp_cond.reg2 as usize] as i32)
            {
                let pc = jmp_cond.pc as i64;
                let pc = pc.wrapping_add(jmp_cond.imm as i64);

                (pc as u32, true)
            } else {
                (0, false)
            }
        }
        JumpCond::LessThanUnsigned => {
            if cpu.regs[jmp_cond.reg1 as usize] < cpu.regs[jmp_cond.reg2 as usize] {
                let pc = jmp_cond.pc as i64;
                let pc = pc.wrapping_add(jmp_cond.imm as i64);

                (pc as u32, true)
            } else {
                (0, false)
            }
        }
        JumpCond::GreaterThanEqualUnsigned => {
            if cpu.regs[jmp_cond.reg1 as usize] >= cpu.regs[jmp_cond.reg2 as usize] {
                let pc = jmp_cond.pc as i64;
                let pc = pc.wrapping_add(jmp_cond.imm as i64);

                (pc as u32, true)
            } else {
                (0, false)
            }
        }
    };

    if !should_jmp {
        return 0;
    }

    let bus = bus::get_bus();

    let jmp_addr = bus.translate(jmp_addr as BusType);

    if jmp_addr.is_err() {
        cpu.bus_error = jmp_addr.err().unwrap();

        ReturnableImpl::throw();
    }

    let jmp_addr = jmp_addr.unwrap();

    let host_addr = cpu.insn_map.get_by_value(jmp_addr);

    if host_addr.is_none() {
        cpu.bus_error = BusError::ForwardJumpFault(jmp_cond.pc);

        ReturnableImpl::throw();
    }

    if jmp_cond.reg1 != 0
        && (jmp_cond.cond == JumpCond::Always || jmp_cond.cond == JumpCond::AlwaysAbsolute)
    {
        cpu.regs[jmp_cond.reg1 as usize] = jmp_cond.pc;
    }

    *host_addr.unwrap()
}

fn emit_jmp(mut cond: JumpVars) -> HostEncodedInsn {
    let mut insn = BackendCoreImpl::emit_usize_call_with_1_arg(jump_resolver_cb, cond.to_usize());

    let cpu = cpu::get_cpu();
    cond.pc = cpu.pc;

    emit_cmp_reg_imm!(insn, amd64_reg::RAX, 0);

    let mut jmp_insn = HostEncodedInsn::new();
    emit_jmp_reg!(jmp_insn, amd64_reg::RAX);

    emit_jz_imm!(insn, jmp_insn.size() as u8 + 1);
    insn.push_slice(jmp_insn.iter().as_slice());

    insn
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
        let insn = emit_bus_access!(rd, rs1, 1, imm, false, true);

        Ok(insn)
    }

    fn emit_lh(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rd, rs1, 2, imm, false, true);

        Ok(insn)
    }

    fn emit_lw(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rd, rs1, 4, imm, false, true);

        Ok(insn)
    }

    fn emit_lbu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rd, rs1, 1, imm, false, false);

        Ok(insn)
    }

    fn emit_lhu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rd, rs1, 2, imm, false, false);

        Ok(insn)
    }

    fn emit_sb(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rs1, rs2, 1, imm, true, true);

        Ok(insn)
    }

    fn emit_sh(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rs1, rs2, 2, imm, true, true);

        Ok(insn)
    }

    fn emit_sw(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        let insn = emit_bus_access!(rs1, rs2, 4, imm, true, true);

        Ok(insn)
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

        match cpu.mode {
            PrivMode::User => {
                emit_set_exception!(insn, cpu, Exception::EnvironmentCallFromUMode);
            }
            PrivMode::Supervisor => {
                emit_set_exception!(insn, cpu, Exception::EnvironmentCallFromSMode);
            }
            PrivMode::Machine => {
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
