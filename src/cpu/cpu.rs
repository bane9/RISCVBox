use crate::backend::PtrT;
use crate::bus::bus::BusType;
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
            _ => panic!("Invalid instruction type: {:#x}", val),
        }
    }
}

pub enum PrivMode {
    User = 0,
    Supervisor = 1,
    Machine = 3,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum RunState {
    None = 0,
    Running = 1,
    Exception = 2,
    BlockExit = 3,
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

pub type CpuReg = BusType;

pub struct Cpu {
    pub pc: CpuReg,
    pub regs: [CpuReg; 32],
    pub csr: [CpuReg; 4096],
    pub mode: PrivMode,
    pub insn_map: BiMap<PtrT, CpuReg>,
    pub missing_insn_map: HashMap<CpuReg, PtrT>,
    pub run_state: RunState,
    pub ret_status: usize,
}

impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            pc: 0,
            regs: [0; 32],
            csr: [0; 4096],
            mode: PrivMode::Machine,
            insn_map: BiMap::new(),
            missing_insn_map: HashMap::new(),
            run_state: RunState::None,
            ret_status: 0,
        }
    }
}

thread_local! {
    static CPU: RefCell<Cpu> = RefCell::new(Cpu::new());
}

pub fn get_cpu() -> &'static mut Cpu {
    CPU.with(|cpu| unsafe { &mut *cpu.as_ptr() })
}
