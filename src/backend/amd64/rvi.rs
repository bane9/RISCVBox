use crate::backend::core::{
    abi_reg, CMP_JMP_IMM32_SIZE, FASTMEM_BLOCK_SIZE, JMP_IMM32_SIZE, MMU_IS_ACTIVE_REG,
};
use crate::backend::target::core::{amd64_reg, BackendCore, BackendCoreImpl};
use crate::backend::{common, ReturnableHandler, ReturnableImpl};
use crate::bus::mmu::AccessType;
use crate::cpu::{CpuReg, JumpAddrPatch};
use crate::frontend::exec_core::{INSN_SIZE, RV_PAGE_SHIFT, RV_PAGE_SIZE};
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

    emit_jz_imm!(insn, jmp_insn.size());
    insn.push_slice(jmp_insn.as_slice());

    insn
}

fn emit_jmp_relative(jump_cond: JumpCond, reg1: CpuReg, reg2: CpuReg, imm: i32) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();
    let diff = cpu.current_gpfn_offset as i32 + imm;

    assert!(diff >= 0 && diff < RV_PAGE_SIZE as i32);

    let target_guest_pc = cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg;
    let target_guest_pc = target_guest_pc as i64 + diff as i64;
    let jmp_insn_offset: u32;

    let mut insn = HostEncodedInsn::new();
    let target_host_addr: usize;

    if jump_cond == JumpCond::Always {
        if reg1 != 0 {
            let auipc = RviImpl::emit_auipc(reg1 as u8, INSN_SIZE as i32).unwrap();

            insn.push_slice(auipc.as_slice());
        }

        target_host_addr = cpu.jit_current_ptr as usize + insn.size();

        emit_jmp_imm32!(insn, 0);
        jmp_insn_offset = JMP_IMM32_SIZE as u32;
    } else {
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RAX, reg1);
        emit_mov_reg_guest_to_host!(insn, cpu, amd64_reg::RBX, reg2);

        emit_cmp_reg_reg32!(insn, amd64_reg::RAX, amd64_reg::RBX);

        target_host_addr = cpu.jit_current_ptr as usize + insn.size();

        match jump_cond {
            JumpCond::Equal => emit_je_imm!(insn, 0),
            JumpCond::NotEqual => emit_jne_imm!(insn, 0),
            JumpCond::LessThan => emit_jl_imm!(insn, 0),
            JumpCond::GreaterThanEqual => emit_jge_imm!(insn, 0),
            JumpCond::LessThanUnsigned => emit_jb_imm!(insn, 0),
            JumpCond::GreaterThanEqualUnsigned => emit_jae_imm!(insn, 0),
            _ => unreachable!(),
        }

        jmp_insn_offset = CMP_JMP_IMM32_SIZE as u32;
    }

    cpu.insn_patch_list.push(JumpAddrPatch::new(
        target_guest_pc as CpuReg,
        target_host_addr as *mut u8,
        jmp_insn_offset,
    ));

    insn
}

fn emit_jmp(jump_cond: JumpCond, reg1: CpuReg, reg2: CpuReg, imm: i32) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    let diff = cpu.current_gpfn_offset as i32 + imm;

    if jump_cond != JumpCond::AlwaysAbsolute
        && diff % INSN_SIZE as i32 == 0
        && diff >= 0
        && diff < RV_PAGE_SIZE as i32
    {
        return emit_jmp_relative(jump_cond, reg1, reg2, imm);
    }

    let jmp_fn = match jump_cond {
        JumpCond::Equal => c_beq_cb,
        JumpCond::NotEqual => c_bne_cb,
        JumpCond::LessThan => c_blt_cb,
        JumpCond::GreaterThanEqual => c_bge_cb,
        JumpCond::LessThanUnsigned => c_bltu_cb,
        JumpCond::GreaterThanEqualUnsigned => c_bgeu_cb,
        JumpCond::Always => c_jal_cb,
        JumpCond::AlwaysAbsolute => c_jalr_cb,
    };

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

extern "C" fn c_load_mmu_translate_cb(addr: usize, gpfn_offset: usize) -> usize {
    let cpu = cpu::get_cpu();

    let bus = bus::get_bus();

    let addr = addr as CpuReg;

    let ret = bus.translate(addr, &cpu.mmu, AccessType::Load);

    if ret.is_err() {
        cpu.set_exception(ret.err().unwrap(), gpfn_offset as CpuReg);

        ReturnableImpl::throw();
    }

    ret.unwrap() as usize
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

    emit_mov_reg_imm_auto!(insn, amd64_reg::RAX, fmem_encoded);
    emit_mov_reg_imm_auto!(insn, amd64_reg::RBX, dst as usize);
    emit_mov_reg_imm_auto!(insn, amd64_reg::RAX, src as usize);
    emit_mov_reg_imm_auto!(insn, amd64_reg::RCX, imm as i64 as usize);

    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);
    emit_add_reg_reg!(insn, amd64_reg::RAX, amd64_reg::RCX);

    let mut mmu_translate_insn = HostEncodedInsn::new();
    emit_mov_reg_reg1!(mmu_translate_insn, abi_reg::ARG1, amd64_reg::RAX);
    emit_mov_reg_imm_auto!(mmu_translate_insn, abi_reg::ARG2, cpu.current_gpfn_offset);
    emit_mov_reg_imm_auto!(
        mmu_translate_insn,
        amd64_reg::R11,
        c_load_mmu_translate_cb as usize
    ); // Stack manipulation is left out as it's technically not needed here
    emit_call_reg!(mmu_translate_insn, amd64_reg::R11);

    emit_cmp_reg_imm!(insn, MMU_IS_ACTIVE_REG, 1);
    emit_jne_imm!(insn, mmu_translate_insn.size());
    insn.push_slice(mmu_translate_insn.as_slice());

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

extern "C" fn c_store_mmu_translate_cb(addr: usize, gpfn_offset: usize) -> usize {
    let cpu = cpu::get_cpu();

    let bus = bus::get_bus();

    let addr = addr as CpuReg;

    let ret = bus.translate(addr, &cpu.mmu, AccessType::Store);

    if ret.is_err() {
        cpu.set_exception(ret.err().unwrap(), gpfn_offset as CpuReg);

        ReturnableImpl::throw();
    }

    ret.unwrap() as usize
}

fn emit_store(store_size: usize, addr_reg: u8, data_reg: u8, imm: i32) -> HostEncodedInsn {
    let cpu = cpu::get_cpu();

    let data = &cpu.regs[data_reg as usize] as *const CpuReg as *mut u8;
    let addr = &cpu.regs[addr_reg as usize] as *const CpuReg as *mut u8;

    let mut insn = HostEncodedInsn::new();

    let fmem_type = FastmemAccessType::Store.to_usize();
    let fmem_encoded = create_fastmem_metadata!(store_size, fmem_type);

    emit_mov_reg_imm_auto!(insn, amd64_reg::RAX, fmem_encoded);
    emit_mov_reg_imm_auto!(insn, amd64_reg::RAX, addr as usize);
    emit_mov_reg_imm_auto!(insn, amd64_reg::RBX, data as usize);
    emit_mov_reg_imm_auto!(insn, amd64_reg::RCX, imm as i64 as usize);

    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);
    emit_add_reg_imm!(insn, amd64_reg::RAX, imm as i64 as usize);

    emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RBX, amd64_reg::RBX);

    let mut mmu_translate_insn = HostEncodedInsn::new();
    emit_mov_reg_reg1!(mmu_translate_insn, abi_reg::ARG1, amd64_reg::RAX);
    emit_mov_reg_imm_auto!(mmu_translate_insn, abi_reg::ARG2, cpu.current_gpfn_offset);
    emit_mov_reg_imm_auto!(
        mmu_translate_insn,
        amd64_reg::R11,
        c_store_mmu_translate_cb as usize
    );
    emit_call_reg!(mmu_translate_insn, amd64_reg::R11);

    emit_cmp_reg_imm!(insn, MMU_IS_ACTIVE_REG, 1);
    emit_jne_imm!(insn, mmu_translate_insn.size());
    insn.push_slice(mmu_translate_insn.as_slice());

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
            emit_mov_reg_imm_auto!(insn, amd64_reg::RBX, imm);
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

        emit_mov_reg_imm_auto!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_imm!(insn, amd64_reg::RBX, imm as u32);

        Ok(insn)
    }

    fn emit_auipc(rd: u8, imm: i32) -> DecodeRet {
        let mut insn = HostEncodedInsn::new();
        let cpu = cpu::get_cpu();

        emit_check_rd!(insn, rd);

        let current_gpfn = &cpu.current_gpfn as *const _ as usize;

        emit_mov_reg_imm_auto!(insn, amd64_reg::RAX, current_gpfn);
        emit_mov_ptr_reg_dword_ptr!(insn, amd64_reg::RAX, amd64_reg::RAX);

        emit_shl_reg_imm!(insn, amd64_reg::RAX, RV_PAGE_SHIFT as u8);

        emit_or_reg_imm!(insn, amd64_reg::RAX, cpu.current_gpfn_offset);

        if imm != 0 {
            emit_add_reg_imm!(insn, amd64_reg::RAX, imm);
        }

        let rd_addr = &cpu.regs[rd as usize] as *const _ as usize;

        emit_mov_reg_imm_auto!(insn, amd64_reg::RBX, rd_addr);
        emit_mov_dword_ptr_reg!(insn, amd64_reg::RBX, amd64_reg::RAX);

        Ok(insn)
    }

    fn emit_jal(rd: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpCond::Always, rd as u32, 0, imm))
    }

    fn emit_jalr(rd: u8, rs1: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(
            JumpCond::AlwaysAbsolute,
            rd as u32,
            rs1 as u32,
            imm,
        ))
    }

    fn emit_beq(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpCond::Equal, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_bne(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpCond::NotEqual, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_blt(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(JumpCond::LessThan, rs1 as u32, rs2 as u32, imm))
    }

    fn emit_bge(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(
            JumpCond::GreaterThanEqual,
            rs1 as u32,
            rs2 as u32,
            imm,
        ))
    }

    fn emit_bltu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(
            JumpCond::LessThanUnsigned,
            rs1 as u32,
            rs2 as u32,
            imm,
        ))
    }

    fn emit_bgeu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet {
        Ok(emit_jmp(
            JumpCond::GreaterThanEqualUnsigned,
            rs1 as u32,
            rs2 as u32,
            imm,
        ))
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
