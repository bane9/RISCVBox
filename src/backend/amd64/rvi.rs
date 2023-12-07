use crate::backend::common;
use crate::backend::core::FASTMEM_BLOCK_SIZE;
use crate::backend::target::core::{amd64_reg, BackendCore, BackendCoreImpl};
use crate::cpu::CpuReg;
use crate::frontend::exec_core::{RV_PAGE_SHIFT, RV_PAGE_SIZE};
use crate::*;
use common::*;

use super::core::FastmemAccessType;

pub struct RviImpl;

fn emit_jmp_absolute(
    jmp_fn: extern "C" fn(usize, usize, usize, usize) -> usize,
    reg1: CpuReg,
    reg2: CpuReg,
    imm: i32,
) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    let mut insn = BackendCoreImpl::emit_usize_call_with_4_args(
        jmp_fn,
        &cpu.regs[reg1 as usize] as *const CpuReg as usize,
        &cpu.regs[reg2 as usize] as *const CpuReg as usize,
        imm as i64 as usize,
        cpu.current_gpfn_offset as usize,
    );

    emit_cmp_reg_imm!(insn, amd64_reg::RAX, 0);

    let mut jmp_insn = HostEncodedInsn::new();
    emit_jmp_reg!(jmp_insn, amd64_reg::RAX);

    emit_jz_imm!(insn, jmp_insn.size() as u8);
    insn.push_slice(jmp_insn.iter().as_slice());

    insn
}

fn emit_jmp(
    jmp_fn: extern "C" fn(usize, usize, usize, usize) -> usize,
    reg1: CpuReg,
    reg2: CpuReg,
    imm: i32,
) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    let diff = cpu.current_gpfn_offset as i32 + imm;

    if diff >= 0 && diff < RV_PAGE_SIZE as i32 {
        //println!("emit_jmp: relative jump");
    }

    emit_jmp_absolute(jmp_fn, reg1, reg2, imm)
}

pub fn emit_bus_access_raw(
    bus_fn: extern "C" fn(usize, usize, usize, usize),
    reg1: *mut u8,
    reg2: *mut u8,
    imm: i32,
    current_gpfn_offset: usize,
) -> HostEncodedInsn {
    BackendCoreImpl::emit_void_call_with_4_args(
        bus_fn,
        reg1 as usize,
        reg2 as usize,
        imm as i64 as usize,
        current_gpfn_offset,
    )
}

fn emit_bus_access(
    bus_fn: extern "C" fn(usize, usize, usize, usize),
    reg1: u8,
    reg2: u8,
    imm: i32,
) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    emit_bus_access_raw(
        bus_fn,
        &cpu.regs[reg1 as usize] as *const CpuReg as *mut u8,
        &cpu.regs[reg2 as usize] as *const CpuReg as *mut u8,
        imm,
        cpu.current_gpfn_offset as usize,
    )
}

fn emit_load(
    load_size: usize,
    dest_reg: u8,
    src_reg: u8,
    imm: i32,
    is_unsigned: bool,
) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    let dst = &cpu.regs[dest_reg as usize] as *const CpuReg as *mut u8;
    let src = &cpu.regs[src_reg as usize] as *const CpuReg as *mut u8;

    let mut insn = HostEncodedInsn::new();

    let fmem_type = if is_unsigned {
        FastmemAccessType::LoadUnsigned.to_usize()
    } else {
        FastmemAccessType::Load.to_usize()
    };

    let fmem_encoded = create_fastmem_metadata!(load_size, fmem_type);

    emit_mov_reg_imm!(insn, amd64_reg::RAX, fmem_encoded);
    emit_mov_reg_imm!(insn, amd64_reg::RBX, dst as usize);
    emit_mov_reg_imm!(insn, amd64_reg::RAX, src as usize);
    emit_mov_reg_imm!(insn, amd64_reg::RCX, imm as i64 as usize);

    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);
    emit_add_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RCX);
    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);

    if dest_reg != 0 {
        match load_size {
            8 => emit_and_reg_imm!(insn, amd64_reg::RAX, 0xff),
            16 => emit_and_reg_imm!(insn, amd64_reg::RAX, 0xffff),
            32 => {}
            _ => panic!("emit_load: invalid load size"),
        }

        if !is_unsigned {
            match load_size {
                8 => emit_movsxd_reg64_reg8!(insn, amd64_reg::RAX, amd64_reg::RAX),
                16 => emit_movsxd_reg64_reg16!(insn, amd64_reg::RAX, amd64_reg::RAX),
                _ => {}
            }
        }

        emit_mov_dword_ptr_reg!(insn, amd64_reg::RBX, amd64_reg::RAX);
    }

    assert!(insn.size() <= FASTMEM_BLOCK_SIZE);

    let diff = FASTMEM_BLOCK_SIZE - insn.size();

    for _ in 0..diff {
        emit_nop!(insn);
    }

    insn
}

fn emit_store(store_size: usize, addr_reg: u8, data_reg: u8, imm: i32) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    let data = &cpu.regs[data_reg as usize] as *const CpuReg as *mut u8;
    let addr = &cpu.regs[addr_reg as usize] as *const CpuReg as *mut u8;

    let mut insn = HostEncodedInsn::new();

    let fmem_type = FastmemAccessType::Store.to_usize();
    let fmem_encoded = create_fastmem_metadata!(store_size, fmem_type);

    emit_mov_reg_imm!(insn, amd64_reg::RAX, fmem_encoded);
    emit_mov_reg_imm!(insn, amd64_reg::RAX, addr as usize);
    emit_mov_reg_imm!(insn, amd64_reg::RBX, data as usize);
    emit_mov_reg_imm!(insn, amd64_reg::RCX, imm as i64 as usize);

    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);
    emit_add_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RCX);

    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RBX, amd64_reg::RBX);

    match store_size {
        8 => emit_mov_byte_ptr_reg!(insn, amd64_reg::RAX, amd64_reg::RBX),
        16 => emit_mov_word_ptr_reg!(insn, amd64_reg::RAX, amd64_reg::RBX),
        32 => emit_mov_dword_ptr_reg!(insn, amd64_reg::RAX, amd64_reg::RBX),
        _ => panic!("emit_store: invalid store size"),
    }

    assert!(insn.size() <= FASTMEM_BLOCK_SIZE);

    let diff = FASTMEM_BLOCK_SIZE - insn.size();

    for _ in 0..diff {
        emit_nop!(insn);
    }

    insn
}

impl common::Rvi for RviImpl {
    fn emit_addi(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        if rs1 == 0 {
            emit_mov_reg_imm!(insn, amd64_reg::RBX, imm);
            emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

            return Ok(insn);
        } else if imm == 0 {
            emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
            emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

            return Ok(insn);
        }

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);

        emit_add_reg_imm!(insn, amd64_reg::RBX, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

        Ok(insn)
    }

    fn emit_add(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        if rs1 == 0 {
            emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs2);
            emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

            return Ok(insn);
        } else if rs2 == 0 {
            emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
            emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

            return Ok(insn);
        }

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

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);

        emit_test_less_reg_imm!(insn, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_sltiu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);

        emit_test_less_reg_uimm!(insn, imm);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_xori(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);

        emit_xor_reg_imm!(insn, amd64_reg::RAX, imm);

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

        emit_movsxd_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);

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

        emit_mov_reg_imm!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_imm!(insn, amd64_reg::RBX, imm as u32);

        Ok(insn)
    }

    fn emit_auipc(rd: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let current_gpfn = &cpu.current_gpfn as *const _ as usize;

        emit_mov_reg_imm!(insn, amd64_reg::RAX, current_gpfn);
        emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);

        emit_shl_reg_imm!(insn, amd64_reg::RAX, RV_PAGE_SHIFT as u8);

        emit_or_reg_imm!(insn, amd64_reg::RAX, cpu.current_gpfn_offset);
        emit_add_reg_imm!(insn, amd64_reg::RAX, imm);

        let rd_addr = &cpu.regs[rd as usize] as *const _ as usize;

        emit_mov_reg_imm!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_reg!(insn, amd64_reg::RBX, amd64_reg::RAX);

        Ok(insn)
    }

    fn emit_jal(rd: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_jal_cb, rd as u32, 0, imm))
    }

    fn emit_jalr(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp_absolute(c_jalr_cb, rd as u32, rs1 as u32, imm))
    }

    fn emit_beq(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_beq_cb, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_bne(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_bne_cb, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_blt(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_blt_cb, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_bge(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_bge_cb, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_bltu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_bltu_cb, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_bgeu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(c_bgeu_cb, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_lb(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_load(8, rd, rs1, imm, false))
    }

    fn emit_lh(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_load(16, rd, rs1, imm, false))
    }

    fn emit_lw(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_load(32, rd, rs1, imm, false))
    }

    fn emit_lbu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_load(8, rd, rs1, imm, true))
    }

    fn emit_lhu(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_load(16, rd, rs1, imm, true))
    }

    fn emit_sb(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_store(8, rs1, rs2, imm))
    }

    fn emit_sh(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_store(16, rs1, rs2, imm))
    }

    fn emit_sw(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_store(32, rs1, rs2, imm))
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

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_movsxd_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX);

        emit_sarx_reg_reg!(insn, amd64_reg::RBX, amd64_reg::RBX, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RBX, rd);

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

        emit_movsxd_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RAX);
        emit_movsxd_reg_reg!(insn, amd64_reg::RCX, amd64_reg::RCX);

        emit_test_less_reg_reg!(insn, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }

    fn emit_sltu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, rs1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RCX, rs2);

        emit_test_less_reg_reg!(insn, amd64_reg::RCX);

        emit_mov_reg_host_to_guest!(insn, cpu, amd64_reg::RCX, amd64_reg::RAX, rd);

        Ok(insn)
    }
}
