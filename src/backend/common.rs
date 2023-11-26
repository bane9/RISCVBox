use cpu::Exception;

use crate::bus::mmu::AccessType;
use crate::bus::{bus, BusType};
use crate::cpu::{cpu, CpuReg};
use crate::frontend::exec_core::{INSN_SIZE, RV_PAGE_MASK, RV_PAGE_SHIFT};
use crate::util::EncodedInsn;

use crate::backend::{ReturnableHandler, ReturnableImpl};
use crate::util::util::sign_extend;

#[derive(Debug)]
pub enum JitError {
    InvalidInstruction(u32),
    ReachedBlockBoundary,
    UnknownError,
}

pub type PtrT = *mut u8;
pub type HostInsnT = u8;
pub const HOST_INSN_MAX_SIZE: usize = 96;
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;
pub const JUMP_COUNT_MAX: usize = 0x1000;

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

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CondVars<T> {
    pub cond: T,
    pub imm: i32,
    pub reg1: CpuReg,
    pub reg2: CpuReg,
}

impl<T> CondVars<T>
where
    T: UsizeConversions,
{
    pub fn to_usize(&self, twenty_bit_imm: bool) -> usize {
        let mut ret = 0;

        ret |= (self.cond.to_usize() & 0x7) << 0;
        ret |= (self.reg1 as usize & 0x1f) << 3;
        ret |= (self.reg2 as usize & 0x1f) << 8;
        ret |= (twenty_bit_imm as usize & 0x1) << 13;
        ret |= (self.imm as usize & 0xffffffff) << 16;

        ret
    }

    pub fn from_usize(val: usize) -> Self {
        let cond = T::from_usize((val >> 0) & 0x7);

        let reg1 = ((val >> 3) & 0x1f) as CpuReg;
        let reg2 = ((val >> 8) & 0x1f) as CpuReg;
        let twenty_bit_imm = ((val >> 13) & 0x1) != 0;
        let imm = (val >> 16) & 0xffffffff;
        let imm = if twenty_bit_imm {
            imm as i32
        } else {
            sign_extend(imm as i64, 12) as i32
        };

        Self {
            cond,
            imm,
            reg1,
            reg2,
        }
    }
}

impl<T> CondVars<T> {
    pub fn new(cond: T, imm: i32, reg1: CpuReg, reg2: CpuReg) -> Self {
        Self {
            cond,
            imm,
            reg1,
            reg2,
        }
    }
}

pub type JumpVars = CondVars<JumpCond>;
pub type BusAccessVars = CondVars<BusAccessCond>;

pub extern "C" fn c_jump_resolver_cb(jmp_cond: usize, guest_pc: usize) -> usize {
    let cpu = cpu::get_cpu();
    let jmp_cond = JumpVars::from_usize(jmp_cond);

    let guest_pc = guest_pc as CpuReg;

    cpu.jump_count += 1;

    if cpu.jump_count > JUMP_COUNT_MAX {
        cpu.set_exception(Exception::BookkeepingRet, guest_pc);

        ReturnableImpl::throw();
    }

    let (jmp_addr, should_jmp) = match jmp_cond.cond {
        JumpCond::Always => {
            let pc = guest_pc as i64;
            let pc = pc + jmp_cond.imm as i64;

            (pc, true)
        }
        JumpCond::AlwaysAbsolute => {
            let pc = cpu.regs[jmp_cond.reg2 as usize] as i64;
            let pc = pc + jmp_cond.imm as i64;
            let pc = pc & !1;

            (pc, true)
        }
        JumpCond::Equal => {
            if cpu.regs[jmp_cond.reg1 as usize] as i32 == cpu.regs[jmp_cond.reg2 as usize] as i32 {
                let pc = guest_pc as i64;
                let pc = pc + jmp_cond.imm as i64;

                (pc, true)
            } else {
                (0, false)
            }
        }
        JumpCond::NotEqual => {
            if cpu.regs[jmp_cond.reg1 as usize] as i32 != cpu.regs[jmp_cond.reg2 as usize] as i32 {
                let pc = guest_pc as i64;
                let pc = pc + jmp_cond.imm as i64;

                (pc, true)
            } else {
                (0, false)
            }
        }
        JumpCond::LessThan => {
            if (cpu.regs[jmp_cond.reg1 as usize] as i32) < (cpu.regs[jmp_cond.reg2 as usize] as i32)
            {
                let pc = guest_pc as i64;
                let pc = pc + jmp_cond.imm as i64;

                (pc, true)
            } else {
                (0, false)
            }
        }
        JumpCond::GreaterThanEqual => {
            if (cpu.regs[jmp_cond.reg1 as usize] as i32)
                >= (cpu.regs[jmp_cond.reg2 as usize] as i32)
            {
                let pc = guest_pc as i64;
                let pc = pc + jmp_cond.imm as i64;

                (pc, true)
            } else {
                (0, false)
            }
        }
        JumpCond::LessThanUnsigned => {
            if cpu.regs[jmp_cond.reg1 as usize] < cpu.regs[jmp_cond.reg2 as usize] {
                let pc = guest_pc as i64;
                let pc = pc + jmp_cond.imm as i64;

                (pc, true)
            } else {
                (0, false)
            }
        }
        JumpCond::GreaterThanEqualUnsigned => {
            if cpu.regs[jmp_cond.reg1 as usize] >= cpu.regs[jmp_cond.reg2 as usize] {
                let pc = guest_pc as i64;
                let pc = pc + jmp_cond.imm as i64;

                (pc, true)
            } else {
                (0, false)
            }
        }
    };

    if !should_jmp {
        return 0;
    }

    let bus = bus::get_bus();

    let jmp_addr1 = if jmp_cond.cond == JumpCond::AlwaysAbsolute {
        jmp_addr
    } else {
        let current_pc = cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg;
        let next_pc = (current_pc as i64) + jmp_addr;

        next_pc
    } as CpuReg;

    if jmp_addr1 % INSN_SIZE as CpuReg != 0 {
        cpu.set_exception(
            Exception::InstructionAddressMisaligned(jmp_addr1 as u32),
            guest_pc,
        );

        ReturnableImpl::throw();
    }

    let jmp_addr_phys = bus.translate(jmp_addr1 as BusType, &cpu.mmu, AccessType::Fetch);

    if jmp_addr_phys.is_err() {
        cpu.set_exception(jmp_addr_phys.err().unwrap(), guest_pc);

        ReturnableImpl::throw();
    }

    let jmp_addr_phys = jmp_addr_phys.unwrap();

    let host_addr = cpu.insn_map.get_by_guest_idx(jmp_addr_phys);

    if jmp_cond.reg1 != 0
        && (jmp_cond.cond == JumpCond::Always || jmp_cond.cond == JumpCond::AlwaysAbsolute)
    {
        cpu.regs[jmp_cond.reg1 as usize] =
            cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg | (guest_pc + INSN_SIZE as CpuReg);
    }

    if host_addr.is_none() {
        cpu.set_exception(Exception::ForwardJumpFault(jmp_addr1), guest_pc);

        ReturnableImpl::throw();
    }

    // If we are jumping to a different page (block boundary won't protect us here)
    // we need to update the current_gpfn.
    cpu.current_gpfn = jmp_addr1 >> RV_PAGE_SHIFT as CpuReg;

    host_addr.unwrap().host_ptr as usize
}

pub extern "C" fn c_bus_resolver_cb(bus_vars: usize, guest_pc: usize) {
    let cpu = cpu::get_cpu();
    let bus_vars = BusAccessVars::from_usize(bus_vars);

    let (addres, size, is_load, is_unsigned) = match bus_vars.cond {
        BusAccessCond::LoadByte => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 8, true, false)
        }
        BusAccessCond::LoadHalf => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 16, true, false)
        }
        BusAccessCond::LoadWord => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 32, true, false)
        }
        BusAccessCond::LoadByteUnsigned => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 8, true, true)
        }
        BusAccessCond::LoadHalfUnsigned => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 16, true, true)
        }
        BusAccessCond::StoreByte => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 8, false, false)
        }
        BusAccessCond::StoreHalf => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 16, false, false)
        }
        BusAccessCond::StoreWord => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr + bus_vars.imm as i64;

            (addr as u32, 32, false, false)
        }
    };

    let bus = bus::get_bus();

    let guest_pc = guest_pc as CpuReg;

    if is_load {
        let data = bus.load(addres, size, &cpu.mmu);

        if data.is_err() {
            cpu.set_exception(data.err().unwrap(), guest_pc);

            ReturnableImpl::throw();
        }

        let data_val = data.unwrap();

        let data_val = if !is_unsigned {
            sign_extend(data_val, size as usize) as u32
        } else {
            data_val
        };

        if bus_vars.reg1 != 0 {
            cpu.regs[bus_vars.reg1 as usize] = data_val;
        }
    } else {
        let data = cpu.regs[bus_vars.reg1 as usize];

        let phys_addr = bus.translate(addres, &cpu.mmu, AccessType::Store);

        if phys_addr.is_err() {
            cpu.set_exception(phys_addr.err().unwrap(), guest_pc);

            ReturnableImpl::throw();
        }

        let res = bus.store_nommu(phys_addr.unwrap(), data, size);

        if res.is_err() {
            cpu.set_exception(res.err().unwrap(), guest_pc);

            ReturnableImpl::throw();
        }

        let gpfn = addres & RV_PAGE_MASK as CpuReg;

        if cpu.gpfn_state.contains_gpfn(gpfn) {
            cpu.set_exception(
                Exception::InvalidateJitBlock(gpfn >> RV_PAGE_SHIFT as CpuReg),
                guest_pc,
            );

            ReturnableImpl::throw();
        }
    };
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
    unsafe fn call_jit_ptr(jit_ptr: PtrT);
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
