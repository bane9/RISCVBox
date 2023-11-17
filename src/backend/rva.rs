use std::sync::atomic::{AtomicI32, AtomicU32};

use super::core::amd64_reg;
use super::{core::BackendCoreImpl, BackendCore};
use crate::*;
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

        let _ptr: Result<*mut u8, Exception> = $bus.get_ptr($addr);

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

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let ptr: *mut u8;
    let addr: CpuReg;

    fetch_ptr!(ptr, addr, bus, cpu, rs1, pc);

    if cpu.atomic_reservations.contains(&addr) {
        atomic_store!(ptr, cpu.regs[rs2], aq_rel);

        cpu.atomic_reservations.remove(&addr);

        cpu.regs[rd] = 0;
    } else {
        cpu.regs[rd] = 1;
    }

    0
}

extern "C" fn amoswapw_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;
    let addr2: u32;

    let ptr1: *mut u8;
    let ptr2: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    fetch_ptr!(ptr2, addr2, bus, cpu, rs2, pc);

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;
        let ptr2 = ptr2 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).swap(
                (*ptr2).load(std::sync::atomic::Ordering::Relaxed),
                std::sync::atomic::Ordering::Relaxed,
            ),
            0b01 => (*ptr1).swap(
                (*ptr2).load(std::sync::atomic::Ordering::Acquire),
                std::sync::atomic::Ordering::Acquire,
            ),
            0b10 => (*ptr1).swap(
                (*ptr2).load(std::sync::atomic::Ordering::Acquire),
                std::sync::atomic::Ordering::Release,
            ),
            0b11 => (*ptr1).swap(
                (*ptr2).load(std::sync::atomic::Ordering::Acquire),
                std::sync::atomic::Ordering::AcqRel,
            ),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

extern "C" fn amoadd_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_add(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_add(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_add(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_add(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

extern "C" fn amoxor_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_xor(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_xor(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_xor(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_xor(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

extern "C" fn amoor_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_or(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_or(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_or(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_or(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

extern "C" fn amosub_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_sub(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_sub(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_sub(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_sub(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

extern "C" fn amoand_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_and(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_and(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_and(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_and(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

extern "C" fn amomin_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicI32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_min(data2 as i32, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_min(data2 as i32, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_min(data2 as i32, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_min(data2 as i32, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data as u32;
    }

    0
}

extern "C" fn amomax_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicI32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_max(data2 as i32, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_max(data2 as i32, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_max(data2 as i32, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_max(data2 as i32, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data as u32;
    }

    0
}

extern "C" fn amominu_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_min(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_min(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_min(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_min(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data as u32;
    }

    0
}

extern "C" fn amomaxu_cb(rd: usize, rs1_rs2: usize, aq_rel: usize, pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let bus = bus::get_bus();

    let rs1 = (rs1_rs2 >> 8) & 0b11111;
    let rs2 = rs1_rs2 & 0b11111;

    let addr1: u32;

    let ptr1: *mut u8;

    fetch_ptr!(ptr1, addr1, bus, cpu, rs1, pc);
    let data2 = cpu.regs[rs2];

    unsafe {
        let ptr1 = ptr1 as *mut AtomicU32;

        let data = match aq_rel {
            0b00 => (*ptr1).fetch_max(data2, std::sync::atomic::Ordering::Relaxed),
            0b01 => (*ptr1).fetch_max(data2, std::sync::atomic::Ordering::Acquire),
            0b10 => (*ptr1).fetch_max(data2, std::sync::atomic::Ordering::Release),
            0b11 => (*ptr1).fetch_max(data2, std::sync::atomic::Ordering::AcqRel),
            _ => unreachable!(),
        };

        cpu.regs[rd] = data;
    }

    0
}

impl common::Rva for RvaImpl {
    fn emit_lr_w(rd: u8, rs1: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            lr_w_cb,
            rd as usize,
            rs1 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_sc_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            sc_w_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amoswap_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amoswapw_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amoadd_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amoadd_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amoxor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amoxor_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amoor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amoor_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amoand_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amoand_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amomin_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amomin_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amomax_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amomax_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amominu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amominu_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }

    fn emit_amomaxu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet {
        emit_atomic_access!(BackendCoreImpl::emit_usize_call_with_4_args(
            amomaxu_cb,
            rd as usize,
            (rs1 as usize) << 8 | rs2 as usize,
            (aq as usize) << 1 | rl as usize,
            cpu::get_cpu().pc as usize,
        ))
    }
}
