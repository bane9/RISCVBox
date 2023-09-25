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

        emit_move_reg_imm!(insn, amd64_reg::R11, &cpu.regs[rs1 as usize] as *const _);
        emit_mov_dword_ptr_reg!(insn, amd64_reg::R11, amd64_reg::R11);
        emit_add_reg_imm!(insn, amd64_reg::R11, imm);
        emit_move_reg_imm!(
            insn,
            amd64_reg::R10,
            &cpu.regs[rd as usize] as *const _ as usize
        );
        emit_mov_dword_ptr_reg!(insn, amd64_reg::R10, amd64_reg::R11);

        Ok(insn)
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
        // let mut insn = HostEncodedInsn::new();
        // let cpu = cpu::get_cpu();

        // let rd_addr = &cpu.regs[rd as usize] as *const _ as usize;

        // emit_move_reg_imm!(insn, amd64_reg::R11, rd_addr);
        // emit_mov_dword_ptr_imm!(insn, amd64_reg::R11, imm as u32);

        // Ok(insn)

        let mut insn = HostEncodedInsn::new();

        emit_nop!(insn);

        Ok(insn)
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
        //let insn = emit_bus_access!(rs1, rs2, 4, imm, true, true);

        let mut insn = HostEncodedInsn::new();

        emit_nop!(insn);

        Ok(insn)
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
