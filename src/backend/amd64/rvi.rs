use crate::backend::common;
use crate::backend::target::core::{amd64_reg, BackendCore, BackendCoreImpl};
use crate::bus::bus::*;
use crate::cpu;
use crate::*;
use common::{DecodeRet, HostEncodedInsn};

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

impl common::Rvi for RviImpl {
    fn emit_addi(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        if rd == 0 {
            emit_nop!(insn);
            return Ok(insn);
        }

        if rs1 != 0 {
            emit_move_reg_imm!(insn, amd64_reg::RBX, &cpu.regs[rs1 as usize] as *const _);
            emit_mov_dword_ptr_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);
        } else {
            emit_move_reg_imm!(insn, amd64_reg::RBX, 0);
        }

        emit_add_reg_imm!(insn, amd64_reg::RBX, imm);
        emit_move_reg_imm!(
            insn,
            amd64_reg::RCX,
            &cpu.regs[rd as usize] as *const _ as usize
        );
        emit_mov_dword_ptr_reg!(insn, amd64_reg::RCX, amd64_reg::RBX);

        Ok(insn)
    }

    fn emit_add(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sub(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slli(_rd: u8, _rs1: u8, _shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slti(_rd: u8, _rs1: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_sltiu(_rd: u8, _rs1: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_xori(_rd: u8, _rs1: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_srli(_rd: u8, _rs1: u8, _shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_srai(_rd: u8, _rs1: u8, _shamt: u8) -> DecodeRet {
        todo!()
    }

    fn emit_ori(_rd: u8, _rs1: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_andi(_rd: u8, _rs1: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_lui(rd: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        if rd == 0 {
            emit_nop!(insn);
            return Ok(insn);
        }

        let rd_addr = &cpu.regs[rd as usize] as *const _ as usize;

        emit_move_reg_imm!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_imm!(insn, amd64_reg::RBX, imm << 12 as u32);

        Ok(insn)
    }

    fn emit_auipc(_rd: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_jal(_rd: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_jalr(_rd: u8, _rs1: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_beq(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bne(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_blt(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bge(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bltu(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        todo!()
    }

    fn emit_bgeu(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        todo!()
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

    fn emit_sw(_rs1: u8, _rs2: u8, _imm: i32) -> DecodeRet {
        //let insn = emit_bus_access!(rs1, rs2, 4, imm, true, true);

        let mut insn = HostEncodedInsn::new();

        emit_nop!(insn);

        Ok(insn)
    }

    fn emit_fence(_pred: u8, _succ: u8) -> DecodeRet {
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

    fn emit_xor(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_srl(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sra(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_or(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_and(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sll(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_slt(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_sltu(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }
}
