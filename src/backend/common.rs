use cpu::Exception;

use crate::bus::{bus, BusType};
use crate::cpu::{cpu, CpuReg};
use crate::frontend::exec_core::INSN_SIZE;
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
pub const HOST_INSN_MAX_SIZE: usize = 64; // TODO: check worst case later
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;

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

pub trait PCAccess {
    fn get_pc(&self) -> u32;
    fn set_pc(&mut self, pc: u32);
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct CondVars<T> {
    pub cond: T,
    pub imm: i32,
    pub reg1: CpuReg,
    pub reg2: CpuReg,
    pub pc: BusType,
}

impl<T> PCAccess for CondVars<T> {
    fn get_pc(&self) -> BusType {
        self.pc
    }

    fn set_pc(&mut self, pc: BusType) {
        self.pc = pc;
    }
}

impl<T> UsizeConversions for CondVars<T>
where
    T: UsizeConversions,
{
    fn to_usize(&self) -> usize {
        let mut ret = 0;

        ret |= (self.cond.to_usize() & 0x7) << 0;
        ret |= (self.reg1 as usize & 0x1f) << 3;
        ret |= (self.reg2 as usize & 0x1f) << 8;
        ret |= (self.imm as usize & 0x7fff) << 13;
        ret |= (self.pc as usize & 0x7fffffff) << 32;

        ret
    }

    fn from_usize(val: usize) -> Self {
        let cond = T::from_usize((val >> 0) & 0x7);

        let reg1 = ((val >> 3) & 0x1f) as CpuReg;
        let reg2 = ((val >> 8) & 0x1f) as CpuReg;
        let imm = ((val >> 13) & 0x7fff) as i32;
        let imm = sign_extend(imm, 12) as i32;
        let pc = ((val >> 32) & 0x7fffffff) as BusType;

        Self {
            cond,
            imm,
            reg1,
            reg2,
            pc,
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
            pc: 0,
        }
    }
}

pub type JumpVars = CondVars<JumpCond>;
pub type BusAccessVars = CondVars<BusAccessCond>;

pub extern "C" fn c_jump_resolver_cb(jmp_cond: usize) -> usize {
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
        cpu.exception = jmp_addr.err().unwrap();

        ReturnableImpl::throw();
    }

    let jmp_addr = jmp_addr.unwrap();

    let host_addr = cpu.insn_map.get_by_value(jmp_addr);

    if host_addr.is_none() {
        cpu.exception = Exception::ForwardJumpFault(jmp_cond.pc);

        ReturnableImpl::throw();
    }

    if jmp_cond.reg1 != 0
        && (jmp_cond.cond == JumpCond::Always || jmp_cond.cond == JumpCond::AlwaysAbsolute)
    {
        cpu.regs[jmp_cond.reg1 as usize] = jmp_cond.pc + INSN_SIZE as u32;
    }

    *host_addr.unwrap()
}

pub extern "C" fn c_bus_resolver_cb(bus_vars: usize) {
    let cpu = cpu::get_cpu();
    let bus_vars = BusAccessVars::from_usize(bus_vars);

    let (addres, size, is_load, _is_unsigned) = match bus_vars.cond {
        BusAccessCond::LoadByte => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 8, true, false)
        }
        BusAccessCond::LoadHalf => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 6, true, false)
        }
        BusAccessCond::LoadWord => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 32, true, false)
        }
        BusAccessCond::LoadByteUnsigned => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 8, true, true)
        }
        BusAccessCond::LoadHalfUnsigned => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 16, true, true)
        }
        BusAccessCond::StoreByte => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 8, false, false)
        }
        BusAccessCond::StoreHalf => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 16, false, false)
        }
        BusAccessCond::StoreWord => {
            let addr = cpu.regs[bus_vars.reg2 as usize] as i64;
            let addr = addr.wrapping_add(bus_vars.imm as i64);

            (addr as u32, 32, false, false)
        }
    };

    let bus = bus::get_bus();

    if is_load {
        let data = bus.read(addres, size);

        if data.is_err() {
            cpu.exception = data.err().unwrap();

            ReturnableImpl::throw();
        }

        let data_val = data.unwrap();

        if bus_vars.reg1 != 0 {
            cpu.regs[bus_vars.reg1 as usize] = data_val;
        }
    } else {
        let data = cpu.regs[bus_vars.reg1 as usize];

        let res = bus.write(addres, data, size);

        if res.is_err() {
            cpu.exception = res.err().unwrap();

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
        println!(
            "Error: Encoding mismatch at \x1b[33m{}\x1b[0m",
            insn_name[28..].trim().replace(" :: ", "::")
        );

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
    fn fill_with_target_nop(ptr: PtrT, size: usize);
    fn fill_with_target_ret(ptr: PtrT, size: usize);
    fn emit_ret() -> HostEncodedInsn;
    fn emit_ret_with_exception(exception: Exception) -> HostEncodedInsn;
    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn;
    fn find_guest_pc_from_host_stack_frame(caller_ret_addr: *mut u8) -> Option<u32>;
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
}
