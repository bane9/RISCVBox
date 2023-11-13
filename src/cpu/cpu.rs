use crate::backend::PtrT;
use crate::bus::bus::BusType;
use crate::bus::BusError;
use crate::cpu::csr;
use crate::util::BiMap;
use std::cell::RefCell;
use std::collections::HashMap;

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
            _ => OpType::Unknown,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RunState {
    None = 0,
    Running = 1,
    Exception = 2,
    BlockExit = 3,
    InvalidInstruction = 4,
    Unknown = 0xff,
}

impl RunState {
    pub fn from_usize(val: usize) -> RunState {
        match val {
            0 => RunState::None,
            1 => RunState::Running,
            2 => RunState::Exception,
            3 => RunState::BlockExit,
            _ => RunState::Unknown,
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

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Exception {
    InstructionAddressMisaligned = 0,
    InstructionAccessFault = 1,
    IllegalInstruction = 2,
    Breakpoint = 3,
    LoadAddressMisaligned = 4,
    LoadAccessFault = 5,
    StoreAddressMisaligned = 6,
    StoreAccessFault = 7,
    EnvironmentCallFromUMode = 8,
    EnvironmentCallFromSMode = 9,
    EnvironmentCallFromMMode = 11,
    InstructionPageFault = 12,
    LoadPageFault = 13,
    StorePageFault = 15,
    None = 0xff,
}

impl Exception {
    pub fn from_usize(val: usize) -> Exception {
        match val {
            0 => Exception::InstructionAddressMisaligned,
            1 => Exception::InstructionAccessFault,
            2 => Exception::IllegalInstruction,
            3 => Exception::Breakpoint,
            4 => Exception::LoadAddressMisaligned,
            5 => Exception::LoadAccessFault,
            6 => Exception::StoreAddressMisaligned,
            7 => Exception::StoreAccessFault,
            8 => Exception::EnvironmentCallFromUMode,
            9 => Exception::EnvironmentCallFromSMode,
            11 => Exception::EnvironmentCallFromMMode,
            12 => Exception::InstructionPageFault,
            13 => Exception::LoadPageFault,
            15 => Exception::StorePageFault,
            _ => Exception::None,
        }
    }
}

pub type CpuReg = BusType;

pub struct Cpu {
    pub pc: CpuReg,
    pub regs: [CpuReg; 32],
    pub insn_map: BiMap<usize, CpuReg>,
    pub missing_insn_map: HashMap<CpuReg, PtrT>,
    pub run_state: RunState,
    pub ret_status: usize,
    pub exception: usize,
    pub bus_error: BusError,
    pub mode: csr::MppMode,
    pub csr: &'static mut csr::Csr,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            pc: 0,
            regs: [0; 32],
            insn_map: BiMap::new(),
            missing_insn_map: HashMap::new(),
            run_state: RunState::None,
            ret_status: 0,
            exception: Exception::None as usize,
            bus_error: BusError::None,
            mode: csr::MppMode::Machine,
            csr: csr::get_csr(),
        }
    }
}

thread_local! {
    static CPU: RefCell<Cpu> = RefCell::new(Cpu::new());
}

pub fn get_cpu() -> &'static mut Cpu {
    CPU.with(|cpu| unsafe { &mut *cpu.as_ptr() })
}
