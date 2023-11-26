use std::sync::atomic::{AtomicI32, AtomicU32};

use super::{core::BackendCoreImpl, BackendCore};
use crate::backend::{ReturnableHandler, ReturnableImpl};
use crate::bus::mmu::AccessType;
use crate::frontend::exec_core::{RV_PAGE_MASK, RV_PAGE_SHIFT};
use crate::{
    backend::common,
    bus,
    cpu::{self, CpuReg, Exception},
};
use common::DecodeRet;

pub struct RvaImpl;

macro_rules! fetch_ptr {
    ($ptr: expr, $addr: expr, $bus: expr, $cpu: expr, $reg: expr, $pc: expr) => {{
        $addr = $cpu.regs[$reg];

        if $addr % 4 != 0 {
            // TODO: insn instead of addr needs to be returned
            $cpu.set_exception(Exception::LoadAddressMisaligned($addr), $pc as CpuReg);
            return 1; // 1 is failure, 0 is success
        }

        let phys_addr = $bus.translate($addr, &cpu::get_cpu().mmu, AccessType::Load);

        if phys_addr.is_err() {
            $cpu.set_exception(phys_addr.err().unwrap(), $pc as CpuReg);
            return 1;
        }

        let _ptr: Result<*mut u8, Exception> = $bus.get_ptr(phys_addr.unwrap());

        if _ptr.is_err() {
            $cpu.set_exception(_ptr.err().unwrap(), $pc as CpuReg);
            return 1;
        }

        $ptr = _ptr.unwrap();

        if $ptr.is_null() {
            $cpu.set_exception(Exception::LoadAccessFault($addr), $pc as CpuReg);
            return 1;
        }
    }};
}

macro_rules! atomic_load {
    ($ptr: expr, $aq_rel: expr) => {{
        unsafe {
            let ptr = $ptr as *mut AtomicU32;

            match $aq_rel {
                0b01 => (*ptr).load(std::sync::atomic::Ordering::Acquire),
                _ => (*ptr).load(std::sync::atomic::Ordering::Relaxed),
            }
        }
    }};
}

macro_rules! check_gpfn_write {
    ($cpu: expr, $addr: expr, $pc: expr) => {{
        let gpfn = $addr & RV_PAGE_MASK as CpuReg;

        if $cpu.gpfn_state.contains_gpfn(gpfn) {
            $cpu.set_exception(
                Exception::InvalidateJitBlock(gpfn >> RV_PAGE_SHIFT as CpuReg),
                $pc,
            );

            ReturnableImpl::throw();
        }
    }};
}

macro_rules! atomic_store {
    ($ptr: expr, $data: expr, $aq_rel: expr) => {{
        unsafe {
            let ptr = $ptr as *mut AtomicU32;

            match $aq_rel {
                0b10 => (*ptr).store($data, std::sync::atomic::Ordering::Release),
                _ => (*ptr).store($data, std::sync::atomic::Ordering::Relaxed),
            }
        }
    }};
}

extern "C" fn lr_w_cb(rd: usize, rs1: usize, aq_rel: usize, pc: usize) -> usize {
    if rd == 0 {
        return 0;
    }

    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let ptr: *mut u8;
    let addr: CpuReg;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);

    cpu.regs[rd] = atomic_load!(ptr, aq_rel);

    cpu.atomic_reservations.insert(addr);

    0
}

extern "C" fn sc_w_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let ptr: *mut u8;
    let addr: CpuReg;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);

    if cpu.atomic_reservations.contains(&addr) {
        atomic_store!(ptr, cpu.regs[rs2], aq_rel);

        cpu.atomic_reservations.remove(&addr);

        if rd != 0 {
            cpu.regs[rd] = 0;
        }

        check_gpfn_write!(cpu, addr, pc as CpuReg);
    } else if rd != 0 {
        cpu.regs[rd] = 1;
    }

    0
}

extern "C" fn amoswapw_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let val = cpu.regs[rs2];

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).swap(val, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).swap(val, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).swap(val, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).swap(val, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amoadd_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_add(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_add(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_add(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_add(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amoxor_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_xor(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_xor(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_xor(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_xor(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amoor_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_or(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_or(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_or(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_or(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amosub_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_sub(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_sub(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_sub(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_sub(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amoand_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_and(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_and(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_and(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_and(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amomin_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicI32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_min(data as i32, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_min(data as i32, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_min(data as i32, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_min(data as i32, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data as CpuReg;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amomax_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicI32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_max(data as i32, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_max(data as i32, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_max(data as i32, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_max(data as i32, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data as CpuReg;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amominu_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_min(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_min(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_min(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_min(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data as CpuReg;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

extern "C" fn amomaxu_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0x1f;
    let rs2 = rs1_rs2 & 0x1f;

    let addr: u32;

    let ptr: *mut u8;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);
    let data = cpu.regs[rs2];

    unsafe {
        let ptr = ptr as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr).fetch_max(data, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr).fetch_max(data, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr).fetch_max(data, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr).fetch_max(data, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        if rd != 0 {
            cpu.regs[rd] = data;
        }
    }

    check_gpfn_write!(cpu, addr, pc as CpuReg);

    0
}

impl common::Rva for RvaImpl {
    fn emit_lr_w(rd: u8, rs1: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                lr_w_cb,
                rd as usize,
                rs1 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_sc_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                sc_w_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amoswap_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amoswapw_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amoadd_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amoadd_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amoxor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amoxor_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amoor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amoor_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amoand_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amoand_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amomin_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amomin_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amomax_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amomax_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amominu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amominu_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }

    fn emit_amomaxu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        Ok(BackendCoreImpl::emit_atomic_access(
            BackendCoreImpl::emit_usize_call_with_4_args(
                amomaxu_cb,
                rd as usize,
                (rs1 as usize) << 8 | rs2 as usize,
                (aq as usize) << 1 | rl as usize,
                cpu::get_cpu().current_gpfn_offset as usize,
            ),
        ))
    }
}
