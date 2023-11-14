use crate::bus::bus::BusType;
use crate::cpu::csr;
use crate::frontend::insn_lookup::InsnData;
use std::cell::RefCell;

pub type CpuReg = BusType;

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
        }
    }

    pub fn get_data(&self) -> CpuReg {
        let data = match self {
            Exception::InstructionAddressMisaligned(data) => data,
            Exception::InstructionAccessFault(data) => data,
            Exception::IllegalInstruction(data) => data,
            Exception::Breakpoint => &0,
            Exception::LoadAddressMisaligned(data) => data,
            Exception::LoadAccessFault(data) => data,
            Exception::StoreAddressMisaligned(data) => data,
            Exception::StoreAccessFault(data) => data,
            Exception::EnvironmentCallFromUMode(data) => data,
            Exception::EnvironmentCallFromSMode(data) => data,
            Exception::EnvironmentCallFromMMode(data) => data,
            Exception::InstructionPageFault(data) => data,
            Exception::LoadPageFault(data) => data,
            Exception::StorePageFault(data) => data,
            Exception::None => &0,
            Exception::ForwardJumpFault(data) => data,
            Exception::BlockExit => &0,
        };

        *data
    }
}

pub struct Cpu {
    pub pc: CpuReg,
    pub regs: [CpuReg; 32],
    pub insn_map: InsnData,
    pub exception: Exception,
    pub c_exception: usize,
    pub c_exception_data: usize,
    pub c_exception_pc: usize,
    pub mode: csr::MppMode,
    pub csr: &'static mut csr::Csr,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            pc: 0,
            regs: [0; 32],
            insn_map: InsnData::new(),
            exception: Exception::None,
            c_exception: Exception::None.to_cpu_reg() as usize,
            c_exception_data: 0,
            c_exception_pc: 0,
            mode: csr::MppMode::Machine,
            csr: csr::get_csr(),
        }
    }

    pub fn set_exception(&mut self, exception: Exception, pc: CpuReg) {
        let cpu = get_cpu();

        cpu.exception = exception;
        cpu.c_exception_pc = pc as usize;
    }
}

thread_local! {
    static CPU: RefCell<Cpu> = RefCell::new(Cpu::new());
}

pub fn get_cpu() -> &'static mut Cpu {
    CPU.with(|cpu| unsafe { &mut *cpu.as_ptr() })
}
