use crate::bus::bus::BusType;
use crate::bus::mmu::{Mmu, Sv32Mmu};
use crate::cpu::csr;
use crate::frontend::gpfn_state::GpfnStateSet;
use crate::frontend::insn_lookup::InsnData;
use std::collections::HashSet;

pub type CpuReg = BusType;

pub const CPU_INTC_PHANDLE: u32 = 0x2;
pub const CPU_TIMEBASE_FREQ: u32 = 1000000;

pub enum RegName {
    Zero = 0,
    Ra = 1,
    Sp = 2,
    Gp = 3,
    Tp = 4,
    T0 = 5,
    T1 = 6,
    T2 = 7,
    S0 = 8,
    S1 = 9,
    A0 = 10,
    A1 = 11,
    A2 = 12,
    A3 = 13,
    A4 = 14,
    A5 = 15,
    A6 = 16,
    A7 = 17,
    S2 = 18,
    S3 = 19,
    S4 = 20,
    S5 = 21,
    S6 = 22,
    S7 = 23,
    S8 = 24,
    S9 = 25,
    S10 = 26,
    S11 = 27,
    T3 = 28,
    T4 = 29,
    T5 = 30,
    T6 = 31,
}

impl RegName {
    pub fn from_usize(reg: usize) -> RegName {
        match reg {
            0 => RegName::Zero,
            1 => RegName::Ra,
            2 => RegName::Sp,
            3 => RegName::Gp,
            4 => RegName::Tp,
            5 => RegName::T0,
            6 => RegName::T1,
            7 => RegName::T2,
            8 => RegName::S0,
            9 => RegName::S1,
            10 => RegName::A0,
            11 => RegName::A1,
            12 => RegName::A2,
            13 => RegName::A3,
            14 => RegName::A4,
            15 => RegName::A5,
            16 => RegName::A6,
            17 => RegName::A7,
            18 => RegName::S2,
            19 => RegName::S3,
            20 => RegName::S4,
            21 => RegName::S5,
            22 => RegName::S6,
            23 => RegName::S7,
            24 => RegName::S8,
            25 => RegName::S9,
            26 => RegName::S10,
            27 => RegName::S11,
            28 => RegName::T3,
            29 => RegName::T4,
            30 => RegName::T5,
            31 => RegName::T6,
            _ => panic!("Invalid register {}", reg),
        }
    }
}

pub enum OpType {
    L = 0x03,

    FENCE = 0x0f,

    I = 0x13,
    S = 0x23,
    A = 0x2f,
    R = 0x33,
    U = 0x37,
    B = 0x63,

    JALR = 0x67,
    JAL = 0x6f,

    AUIPC = 0x17,

    CSR = 0x73,

    Unknown = 0x100,
}

impl OpType {
    pub fn from_u32(val: u32) -> OpType {
        match val {
            0x03 => OpType::L,
            0x0f => OpType::FENCE,
            0x13 => OpType::I,
            0x23 => OpType::S,
            0x2f => OpType::A,
            0x33 => OpType::R,
            0x37 => OpType::U,
            0x63 => OpType::B,
            0x67 => OpType::JALR,
            0x6f => OpType::JAL,
            0x73 => OpType::CSR,
            0x17 => OpType::AUIPC,
            _ => OpType::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Interrupt {
    UserSoftware = 0,
    SupervisorSoftware = 1,
    MachineSoftware = 3,
    UserTimer = 4,
    SupervisorTimer = 5,
    MachineTimer = 7,
    UserExternal = 8,
    SupervisorExternal = 9,
    MachineExternal = 11,
    None = 0xff,
}

impl Interrupt {
    pub fn to_cpu_reg(&self) -> CpuReg {
        match self {
            Interrupt::UserSoftware => 0,
            Interrupt::SupervisorSoftware => 1,
            Interrupt::MachineSoftware => 3,
            Interrupt::UserTimer => 4,
            Interrupt::SupervisorTimer => 5,
            Interrupt::MachineTimer => 7,
            Interrupt::UserExternal => 8,
            Interrupt::SupervisorExternal => 9,
            Interrupt::MachineExternal => 11,
            Interrupt::None => 0xff,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Eq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Exception {
    InstructionAddressMisaligned(CpuReg) = 0,
    InstructionAccessFault(CpuReg) = 1,
    IllegalInstruction(CpuReg) = 2,
    Breakpoint = 3,
    LoadAddressMisaligned(CpuReg) = 4,
    LoadAccessFault(CpuReg) = 5,
    StoreAddressMisaligned(CpuReg) = 6,
    StoreAccessFault(CpuReg) = 7,
    EnvironmentCallFromUMode(CpuReg) = 8,
    EnvironmentCallFromSMode(CpuReg) = 9,
    EnvironmentCallFromMMode(CpuReg) = 11,
    InstructionPageFault(CpuReg) = 12,
    LoadPageFault(CpuReg) = 13,
    StorePageFault(CpuReg) = 15,
    None = 0xff,

    ForwardJumpFault(CpuReg) = 0x100,
    BlockExit = 0x101,
    Mret = 0x102,
    Sret = 0x103,
    InvalidateJitBlock(CpuReg, bool) = 0x104,
    DiscardJitBlock(CpuReg) = 0x105,
    MmuStateUpdate = 0x106,
    Wfi = 0x107,
    BookkeepingRet = 0x108,
    FastmemViolation = 0x109,
    Reboot = 0x10a,
}

impl Exception {
    pub fn from_cpu_reg(val: CpuReg, data: CpuReg) -> Exception {
        match val {
            0 => Exception::InstructionAddressMisaligned(data),
            1 => Exception::InstructionAccessFault(data),
            2 => Exception::IllegalInstruction(data),
            3 => Exception::Breakpoint,
            4 => Exception::LoadAddressMisaligned(data),
            5 => Exception::LoadAccessFault(data),
            6 => Exception::StoreAddressMisaligned(data),
            7 => Exception::StoreAccessFault(data),
            8 => Exception::EnvironmentCallFromUMode(data),
            9 => Exception::EnvironmentCallFromSMode(data),
            11 => Exception::EnvironmentCallFromMMode(data),
            12 => Exception::InstructionPageFault(data),
            13 => Exception::LoadPageFault(data),
            15 => Exception::StorePageFault(data),
            0xff => Exception::None,
            0x100 => Exception::ForwardJumpFault(data),
            0x101 => Exception::BlockExit,
            0x102 => Exception::Mret,
            0x103 => Exception::Sret,
            0x104 => Exception::InvalidateJitBlock(data, true),
            0x105 => Exception::DiscardJitBlock(data),
            0x106 => Exception::MmuStateUpdate,
            0x107 => Exception::Wfi,
            0x108 => Exception::BookkeepingRet,
            0x109 => Exception::FastmemViolation,
            0x10a => Exception::Reboot,
            _ => Exception::None,
        }
    }

    pub fn to_cpu_reg(&self) -> CpuReg {
        match self {
            Exception::InstructionAddressMisaligned(_) => 0,
            Exception::InstructionAccessFault(_) => 1,
            Exception::IllegalInstruction(_) => 2,
            Exception::Breakpoint => 3,
            Exception::LoadAddressMisaligned(_) => 4,
            Exception::LoadAccessFault(_) => 5,
            Exception::StoreAddressMisaligned(_) => 6,
            Exception::StoreAccessFault(_) => 7,
            Exception::EnvironmentCallFromUMode(_) => 8,
            Exception::EnvironmentCallFromSMode(_) => 9,
            Exception::EnvironmentCallFromMMode(_) => 11,
            Exception::InstructionPageFault(_) => 12,
            Exception::LoadPageFault(_) => 13,
            Exception::StorePageFault(_) => 15,
            Exception::None => 0xff,
            Exception::ForwardJumpFault(_) => 0x100,
            Exception::BlockExit => 0x101,
            Exception::Mret => 0x102,
            Exception::Sret => 0x103,
            Exception::InvalidateJitBlock(_, _) => 0x104,
            Exception::DiscardJitBlock(_) => 0x105,
            Exception::MmuStateUpdate => 0x106,
            Exception::Wfi => 0x107,
            Exception::BookkeepingRet => 0x108,
            Exception::FastmemViolation => 0x109,
            Exception::Reboot => 0x10a,
        }
    }

    pub fn get_data(&self) -> CpuReg {
        let data = match self {
            Exception::InstructionAddressMisaligned(data) => *data,
            Exception::InstructionAccessFault(data) => *data,
            Exception::IllegalInstruction(data) => *data,
            Exception::Breakpoint => 0,
            Exception::LoadAddressMisaligned(data) => *data,
            Exception::LoadAccessFault(data) => *data,
            Exception::StoreAddressMisaligned(data) => *data,
            Exception::StoreAccessFault(data) => *data,
            Exception::EnvironmentCallFromUMode(data) => *data,
            Exception::EnvironmentCallFromSMode(data) => *data,
            Exception::EnvironmentCallFromMMode(data) => *data,
            Exception::InstructionPageFault(data) => *data,
            Exception::LoadPageFault(data) => *data,
            Exception::StorePageFault(data) => *data,
            Exception::None => 0,
            Exception::ForwardJumpFault(data) => *data,
            Exception::BlockExit => 0,
            Exception::Mret => 0,
            Exception::Sret => 0,
            Exception::InvalidateJitBlock(data, _) => *data,
            Exception::DiscardJitBlock(data) => *data,
            Exception::MmuStateUpdate => 0,
            Exception::Wfi => 0,
            Exception::BookkeepingRet => 0,
            Exception::FastmemViolation => 0,
            Exception::Reboot => 0,
        };

        data
    }
}

pub struct JumpAddrPatch {
    guest_addr: CpuReg,
    host_addr: *mut u8,
    jump_insn_offset: u32,
}

impl JumpAddrPatch {
    pub fn new(guest_addr: CpuReg, host_addr: *mut u8, jump_insn_offset: u32) -> JumpAddrPatch {
        JumpAddrPatch {
            guest_addr,
            host_addr,
            jump_insn_offset,
        }
    }

    pub fn get_guest_addr(&self) -> CpuReg {
        self.guest_addr
    }

    pub fn get_host_addr(&self) -> *mut u8 {
        self.host_addr
    }

    pub fn get_jmp_insn_offset(&self) -> u32 {
        self.jump_insn_offset
    }

    pub fn get_data(&self) -> (CpuReg, *mut u8, u32) {
        (self.guest_addr, self.host_addr, self.jump_insn_offset)
    }
}

pub struct Cpu {
    pub core_id: CpuReg,
    pub next_pc: CpuReg,
    pub current_gpfn: CpuReg,
    pub current_guest_page: CpuReg,
    pub current_gpfn_offset: CpuReg,
    pub regs: [CpuReg; 32],
    pub insn_map: InsnData,
    pub insn_patch_list: Vec<JumpAddrPatch>,
    pub jit_current_ptr: *mut u8,
    pub exception: Exception,
    pub c_exception: usize,
    pub c_exception_data: usize,
    pub c_exception_pc: usize,
    pub jump_count: usize,
    pub mode: csr::MppMode,
    pub gpfn_state: GpfnStateSet,
    pub atomic_reservations: HashSet<BusType>, // TODO: this probably isn't core local, check later
    pub mmu: Sv32Mmu,
    pub csr: &'static mut csr::Csr,
    pub has_pending_interrupt: std::sync::atomic::AtomicU32,
    pub pending_interrupt_number: CpuReg,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            core_id: CpuReg::MAX,
            next_pc: 0,
            current_gpfn: 0,
            current_guest_page: 0,
            current_gpfn_offset: 0,
            regs: [0; 32],
            insn_map: InsnData::new(),
            insn_patch_list: Vec::new(),
            jit_current_ptr: std::ptr::null_mut(),
            exception: Exception::None,
            c_exception: Exception::None.to_cpu_reg() as usize,
            c_exception_data: 0,
            c_exception_pc: 0,
            jump_count: 0,
            mode: csr::MppMode::Machine,
            gpfn_state: GpfnStateSet::new(),
            atomic_reservations: HashSet::new(),
            mmu: Sv32Mmu::new(),
            csr: csr::get_csr(),
            has_pending_interrupt: std::sync::atomic::AtomicU32::new(0),
            pending_interrupt_number: 0,
        }
    }

    pub fn set_exception(&mut self, exception: Exception, pc: CpuReg) {
        let cpu = get_cpu();

        cpu.exception = exception;
        cpu.c_exception_pc = pc as usize;
    }
}

#[thread_local]
static mut CPU: *mut Cpu = std::ptr::null_mut();

static mut PERCPU_LIST: Vec<*mut Cpu> = Vec::new();

pub fn init_cpu() {
    unsafe {
        CPU = Box::into_raw(Box::new(Cpu::new()));

        PERCPU_LIST.push(CPU);
    }
}

pub fn get_cpu() -> &'static mut Cpu {
    unsafe { &mut *CPU }
}

pub fn remove_all_cpus() {
    unsafe {
        for cpu in PERCPU_LIST.iter() {
            let _ = Box::from_raw(*cpu);
        }

        PERCPU_LIST.clear();
    }
}
