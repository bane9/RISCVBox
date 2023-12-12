use cpu::{Exception, JumpAddrPatch};

use crate::bus::bus::{self, BusType};
use crate::bus::mmu::{AccessType, Mmu};
use crate::cpu::{cpu, CpuReg};
use crate::frontend::exec_core::{INSN_SIZE, RV_PAGE_MASK, RV_PAGE_SHIFT};
use crate::util::EncodedInsn;

use crate::backend::{ReturnableHandler, ReturnableImpl};
use crate::util::util::sign_extend;

use super::target;

#[derive(Debug)]
pub enum JitError {
    InvalidInstruction(u32),
    ReachedBlockBoundary,
    UnknownError,
}

pub type PtrT = *mut u8;
pub type HostInsnT = u8;
pub const HOST_INSN_MAX_SIZE: usize = target::core::HOST_INSN_MAX_SIZE;
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;
pub const JUMP_COUNT_MAX: usize = 0x1000 * 0x10;

pub trait UsizeConversions {
    fn to_usize(&self) -> usize;
    fn from_usize(val: usize) -> Self;
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum JumpCond {
    Always,
    AlwaysAbsolute,
    Equal,
    NotEqual,
    LessThan,
    GreaterThanEqual,
    LessThanUnsigned,
    GreaterThanEqualUnsigned,
}

impl UsizeConversions for JumpCond {
    fn to_usize(&self) -> usize {
        match self {
            JumpCond::Always => 0,
            JumpCond::AlwaysAbsolute => 1,
            JumpCond::Equal => 2,
            JumpCond::NotEqual => 3,
            JumpCond::LessThan => 4,
            JumpCond::GreaterThanEqual => 5,
            JumpCond::LessThanUnsigned => 6,
            JumpCond::GreaterThanEqualUnsigned => 7,
        }
    }

    fn from_usize(val: usize) -> JumpCond {
        match val {
            0 => JumpCond::Always,
            1 => JumpCond::AlwaysAbsolute,
            2 => JumpCond::Equal,
            3 => JumpCond::NotEqual,
            4 => JumpCond::LessThan,
            5 => JumpCond::GreaterThanEqual,
            6 => JumpCond::LessThanUnsigned,
            7 => JumpCond::GreaterThanEqualUnsigned,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BusAccessCond {
    LoadByte,
    LoadHalf,
    LoadWord,
    LoadByteUnsigned,
    LoadHalfUnsigned,
    StoreByte,
    StoreHalf,
    StoreWord,
}

impl BusAccessCond {
    fn bit_size(&self) -> usize {
        match self {
            BusAccessCond::LoadByte => 8,
            BusAccessCond::LoadHalf => 16,
            BusAccessCond::LoadWord => 32,
            BusAccessCond::LoadByteUnsigned => 8,
            BusAccessCond::LoadHalfUnsigned => 16,
            BusAccessCond::StoreByte => 8,
            BusAccessCond::StoreHalf => 16,
            BusAccessCond::StoreWord => 32,
        }
    }
}

impl UsizeConversions for BusAccessCond {
    fn to_usize(&self) -> usize {
        match self {
            BusAccessCond::LoadByte => 0,
            BusAccessCond::LoadHalf => 1,
            BusAccessCond::LoadWord => 2,
            BusAccessCond::LoadByteUnsigned => 3,
            BusAccessCond::LoadHalfUnsigned => 4,
            BusAccessCond::StoreByte => 5,
            BusAccessCond::StoreHalf => 6,
            BusAccessCond::StoreWord => 7,
        }
    }

    fn from_usize(val: usize) -> BusAccessCond {
        match val {
            0 => BusAccessCond::LoadByte,
            1 => BusAccessCond::LoadHalf,
            2 => BusAccessCond::LoadWord,
            3 => BusAccessCond::LoadByteUnsigned,
            4 => BusAccessCond::LoadHalfUnsigned,
            5 => BusAccessCond::StoreByte,
            6 => BusAccessCond::StoreHalf,
            7 => BusAccessCond::StoreWord,
            _ => unreachable!(),
        }
    }
}

#[inline(always)]
fn do_jump(guest_address: CpuReg, current_guest_pc: CpuReg, rd: *mut CpuReg) -> usize {
    let cpu = cpu::get_cpu();

    if !rd.is_null() {
        unsafe {
            *rd = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg)
                + (current_guest_pc + INSN_SIZE as CpuReg);
        }
    }

    let guest_address_phys = if cpu.mmu.is_active() {
        let bus = bus::get_bus();

        let guest_address_phys = bus.translate(guest_address, &cpu.mmu, AccessType::Fetch);

        if guest_address_phys.is_err() {
            cpu.set_exception(guest_address_phys.err().unwrap(), current_guest_pc);

            ReturnableImpl::throw();
        }

        guest_address_phys.unwrap()
    } else {
        guest_address
    };

    let host_addr = cpu.insn_map.get_by_guest_idx(guest_address_phys);

    if host_addr.is_none() {
        cpu.set_exception(Exception::ForwardJumpFault(guest_address), current_guest_pc);

        ReturnableImpl::throw();
    }

    // If we are jumping to a different page (block boundary won't protect us here)
    // we need to update the current_gpfn.
    cpu.current_gpfn = guest_address >> RV_PAGE_SHIFT as CpuReg;
    cpu.current_guest_page = guest_address & RV_PAGE_MASK as CpuReg;

    host_addr.unwrap().host_ptr as usize
}

pub extern "C" fn c_jal_cb(rd: usize, _: usize, imm: usize, guest_pc: usize) -> usize {
    let cpu = cpu::get_cpu();

    let pc = guest_pc as i64;
    let pc = pc.wrapping_add(imm as i64);

    let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

    let rd = if rd == &cpu.regs[0] as *const CpuReg as usize {
        std::ptr::null_mut()
    } else {
        rd as *mut CpuReg
    };

    do_jump(pc as CpuReg, guest_pc as CpuReg, rd)
}

pub extern "C" fn c_jalr_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) -> usize {
    let cpu = cpu::get_cpu();

    let pc = unsafe { *(rs1 as *mut CpuReg) as i64 };
    let pc = pc + imm as i64;
    let pc = pc & !1;

    let rd = if rd == &cpu.regs[0] as *const CpuReg as usize {
        std::ptr::null_mut()
    } else {
        rd as *mut CpuReg
    };

    do_jump(pc as CpuReg, guest_pc as CpuReg, rd)
}

pub extern "C" fn c_beq_cb(rs1: usize, rs2: usize, imm: usize, guest_pc: usize) -> usize {
    let result = unsafe {
        let rs1 = *(rs1 as *mut CpuReg);
        let rs2 = *(rs2 as *mut CpuReg);

        rs1 == rs2
    };

    if result {
        let cpu = cpu::get_cpu();

        let pc = guest_pc as i64;
        let pc = pc.wrapping_add(imm as i64);

        let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

        do_jump(pc as CpuReg, guest_pc as CpuReg, std::ptr::null_mut())
    } else {
        0
    }
}

pub extern "C" fn c_bne_cb(rs1: usize, rs2: usize, imm: usize, guest_pc: usize) -> usize {
    let result = unsafe {
        let rs1 = *(rs1 as *mut CpuReg);
        let rs2 = *(rs2 as *mut CpuReg);

        rs1 != rs2
    };

    if result {
        let cpu = cpu::get_cpu();

        let pc = guest_pc as i64;
        let pc = pc.wrapping_add(imm as i64);

        let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

        do_jump(pc as CpuReg, guest_pc as CpuReg, std::ptr::null_mut())
    } else {
        0
    }
}

pub extern "C" fn c_blt_cb(rs1: usize, rs2: usize, imm: usize, guest_pc: usize) -> usize {
    let result = unsafe {
        let rs1 = *(rs1 as *mut CpuReg) as i32;
        let rs2 = *(rs2 as *mut CpuReg) as i32;

        rs1 < rs2
    };

    if result {
        let cpu = cpu::get_cpu();

        let pc = guest_pc as i64;
        let pc = pc.wrapping_add(imm as i64);

        let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

        do_jump(pc as CpuReg, guest_pc as CpuReg, std::ptr::null_mut())
    } else {
        0
    }
}

pub extern "C" fn c_bge_cb(rs1: usize, rs2: usize, imm: usize, guest_pc: usize) -> usize {
    let result = unsafe {
        let rs1 = *(rs1 as *mut CpuReg) as i32;
        let rs2 = *(rs2 as *mut CpuReg) as i32;

        rs1 >= rs2
    };

    if result {
        let cpu = cpu::get_cpu();

        let pc = guest_pc as i64;
        let pc = pc.wrapping_add(imm as i64);

        let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

        do_jump(pc as CpuReg, guest_pc as CpuReg, std::ptr::null_mut())
    } else {
        0
    }
}

pub extern "C" fn c_bltu_cb(rs1: usize, rs2: usize, imm: usize, guest_pc: usize) -> usize {
    let result = unsafe {
        let rs1 = *(rs1 as *mut CpuReg);
        let rs2 = *(rs2 as *mut CpuReg);

        rs1 < rs2
    };

    if result {
        let cpu = cpu::get_cpu();

        let pc = guest_pc as i64;
        let pc = pc.wrapping_add(imm as i64);

        let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

        do_jump(pc as CpuReg, guest_pc as CpuReg, std::ptr::null_mut())
    } else {
        0
    }
}

pub extern "C" fn c_bgeu_cb(rs1: usize, rs2: usize, imm: usize, guest_pc: usize) -> usize {
    let result = unsafe {
        let rs1 = *(rs1 as *mut CpuReg);
        let rs2 = *(rs2 as *mut CpuReg);

        rs1 >= rs2
    };

    if result {
        let cpu = cpu::get_cpu();

        let pc = guest_pc as i64;
        let pc = pc.wrapping_add(imm as i64);

        let pc = (cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg) as i64 + pc;

        do_jump(pc as CpuReg, guest_pc as CpuReg, std::ptr::null_mut())
    } else {
        0
    }
}

macro_rules! do_load {
    ($rs1:expr, $imm:expr, $guest_pc:expr, $load_size:expr) => {{
        let cpu = cpu::get_cpu();

        let addr = unsafe { *$rs1 } as i64;
        let addr = (addr.wrapping_add($imm as i64)) as CpuReg;

        let data = bus::get_bus().load(addr, $load_size as BusType, &cpu.mmu);

        if data.is_err() {
            cpu.set_exception(data.err().unwrap(), $guest_pc as CpuReg);

            ReturnableImpl::throw();
        }

        data.unwrap()
    }};
}

pub extern "C" fn c_lb_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    let val = do_load!(rs1 as *mut CpuReg, imm as i32, guest_pc as CpuReg, 8);

    unsafe {
        *(rd as *mut CpuReg) = sign_extend(val, 8) as CpuReg;
    }
}

pub extern "C" fn c_lh_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    let val = do_load!(rs1 as *mut CpuReg, imm as i32, guest_pc as CpuReg, 16);

    unsafe {
        *(rd as *mut CpuReg) = sign_extend(val, 16) as CpuReg;
    }
}

pub extern "C" fn c_lw_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    let val = do_load!(rs1 as *mut CpuReg, imm as i32, guest_pc as CpuReg, 32);

    unsafe {
        *(rd as *mut CpuReg) = sign_extend(val, 32) as CpuReg;
    }
}

pub extern "C" fn c_lbu_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    let val = do_load!(rs1 as *mut CpuReg, imm as i32, guest_pc as CpuReg, 8);

    unsafe {
        *(rd as *mut CpuReg) = val as CpuReg;
    }
}

pub extern "C" fn c_lhu_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    let val = do_load!(rs1 as *mut CpuReg, imm as i32, guest_pc as CpuReg, 16);

    unsafe {
        *(rd as *mut CpuReg) = val as CpuReg;
    }
}

#[inline(always)]
fn do_store(rs1: *mut CpuReg, rs2: *mut CpuReg, imm: i32, guest_pc: CpuReg, store_size: u8) {
    let cpu = cpu::get_cpu();

    let addr = unsafe { *rs1 } as i64;
    let addr = addr + imm as i64;

    let addr = addr as CpuReg;

    let bus = bus::get_bus();

    let guest_pc = guest_pc as CpuReg;

    let data = unsafe { *rs2 };

    let result = bus.store(addr, data, store_size as BusType, &cpu.mmu);

    if result.is_err() {
        cpu.set_exception(result.err().unwrap(), guest_pc);

        ReturnableImpl::throw();
    }

    // let gpfn = addr & RV_PAGE_MASK as CpuReg;

    // if cpu.gpfn_state.contains_gpfn(gpfn) {
    //     cpu.set_exception(
    //         Exception::InvalidateJitBlock(gpfn >> RV_PAGE_SHIFT as CpuReg),
    //         guest_pc,
    //     );

    //     ReturnableImpl::throw();
    // }
}

pub extern "C" fn c_sb_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    do_store(
        rd as *mut CpuReg,
        rs1 as *mut CpuReg,
        imm as i32,
        guest_pc as CpuReg,
        8,
    );
}

pub extern "C" fn c_sh_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    do_store(
        rd as *mut CpuReg,
        rs1 as *mut CpuReg,
        imm as i32,
        guest_pc as CpuReg,
        16,
    );
}

pub extern "C" fn c_sw_cb(rd: usize, rs1: usize, imm: usize, guest_pc: usize) {
    do_store(
        rd as *mut CpuReg,
        rs1 as *mut CpuReg,
        imm as i32,
        guest_pc as CpuReg,
        32,
    );
}

pub fn test_asm_common(enc: &HostEncodedInsn, expected: &[u8], insn_name: &str) {
    let mut success = true;
    let mut expected_str = String::new();
    let mut encoded_str = String::new();

    expected_str.push_str("");
    encoded_str.push_str("");

    for (_, (a, b)) in enc.iter().zip(expected.iter()).enumerate() {
        if a != b {
            success = false;

            expected_str.push_str(&format!("\x1b[32m{:02x}\x1b[0m ", b));
            encoded_str.push_str(&format!("\x1b[31m{:02x}\x1b[0m ", a));
        } else {
            expected_str.push_str(&format!("{:02x} ", b));
            encoded_str.push_str(&format!("{:02x} ", a));
        }
    }

    for &b in expected.get(enc.size()..).unwrap_or(&[]) {
        success = false;
        expected_str.push_str(&format!("\x1b[32m{:02x}\x1b[0m ", b));
    }

    for &a in enc.iter().skip(expected.len()) {
        success = false;
        encoded_str.push_str(&format!("\x1b[31m{:02x}\x1b[0m ", a));
    }

    if !success {
        println!("__________________________________________________________\n");
        println!("Error: Encoding mismatch at \x1b[33m{}\x1b[0m", insn_name);

        println!("Expected -> {}", expected_str.trim_end());
        println!("Encoded  -> {}", encoded_str.trim_end());
        println!("__________________________________________________________\n");
    }

    assert!(success);
}

#[macro_export]
macro_rules! test_encoded_insn {
    ($test_name:ident, $insn_macro:expr, $expected:expr) => {
        #[test]
        pub fn $test_name() {
            let mut enc = HostEncodedInsn::new();

            $insn_macro(&mut enc);

            test_asm_common(&enc, &$expected, stringify!($test_name));
        }
    };
}

pub trait BackendCore {
    fn emit_atomic_access(insn: HostEncodedInsn) -> HostEncodedInsn;
    fn emit_ret() -> HostEncodedInsn;
    fn emit_nop() -> HostEncodedInsn;
    fn emit_ret_with_exception(exception: Exception) -> HostEncodedInsn;
    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn;
    fn emit_usize_call_with_4_args(
        fn_ptr: extern "C" fn(usize, usize, usize, usize) -> usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
    ) -> HostEncodedInsn;
    fn emit_void_call_with_4_args(
        fn_ptr: extern "C" fn(usize, usize, usize, usize),
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
    ) -> HostEncodedInsn;
    fn emit_usize_call_with_2_args(
        fn_ptr: extern "C" fn(usize, usize) -> usize,
        arg1: usize,
        arg2: usize,
    ) -> HostEncodedInsn;
    fn emit_void_call_with_2_args(
        fn_ptr: extern "C" fn(usize, usize),
        arg1: usize,
        arg2: usize,
    ) -> HostEncodedInsn;
    fn emit_void_call_with_1_arg(fn_ptr: extern "C" fn(usize), arg1: usize) -> HostEncodedInsn;
    fn emit_usize_call_with_1_arg(
        fn_ptr: extern "C" fn(usize) -> usize,
        arg1: usize,
    ) -> HostEncodedInsn;
    fn fastmem_violation_likely_offset() -> usize;
    fn patch_fastmem_violation(host_exception_addr: usize, guest_exception_addr: BusType);
    fn patch_jump_list(jump_list: &Vec<JumpAddrPatch>);
    unsafe fn call_jit_ptr(jit_ptr: PtrT);
    unsafe fn call_jit_ptr_nommu(jit_ptr: PtrT);
}

pub trait Rvi {
    fn emit_addi(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_add(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sub(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_slli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_slti(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_sltiu(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_xori(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_srli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_srai(rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_ori(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_andi(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_xor(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_srl(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sra(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_or(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_and(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sll(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_slt(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sltu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;

    fn emit_lui(rd: u8, imm: i32) -> DecodeRet;
    fn emit_auipc(rd: u8, imm: i32) -> DecodeRet;
    fn emit_jal(rd: u8, imm: i32) -> DecodeRet;
    fn emit_jalr(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_beq(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bne(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_blt(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bge(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bltu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bgeu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;

    fn emit_lb(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lh(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lw(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lbu(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lhu(rd: u8, rs1: u8, imm: i32) -> DecodeRet;

    fn emit_sb(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_sh(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_sw(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;

    fn emit_fence(pred: u8, succ: u8) -> DecodeRet;
    fn emit_fence_i() -> DecodeRet;
}

pub trait Rvm {
    fn emit_mul(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulh(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulhsu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulhu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;

    fn emit_div(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_divu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_rem(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_remu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
}

pub trait Rva {
    fn emit_lr_w(rd: u8, rs1: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_sc_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoswap_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoadd_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoxor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoand_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomin_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomax_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amominu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomaxu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;
}

pub trait Csr {
    fn emit_csrrw(rd: u8, rs1: u8, csr: u16) -> DecodeRet;
    fn emit_csrrs(rd: u8, rs1: u8, csr: u16) -> DecodeRet;
    fn emit_csrrc(rd: u8, rs1: u8, csr: u16) -> DecodeRet;
    fn emit_csrrwi(rd: u8, zimm: u8, csr: u16) -> DecodeRet;
    fn emit_csrrsi(rd: u8, zimm: u8, csr: u16) -> DecodeRet;
    fn emit_csrrci(rd: u8, zimm: u8, csr: u16) -> DecodeRet;

    fn emit_ecall() -> DecodeRet;
    fn emit_ebreak() -> DecodeRet;
    fn emit_sret() -> DecodeRet;
    fn emit_mret() -> DecodeRet;

    fn emit_wfi() -> DecodeRet;

    fn emit_sfence_vma() -> DecodeRet;
}
