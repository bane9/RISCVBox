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

pub struct Cpu {
    pub pc: u64,
    pub regs: [u64; 32],
    pub csr: [u64; 4096],
    pub mem: *mut u8,
    pub mode: PrivMode,
}

impl Cpu {
    pub fn new(mem: *mut u8) -> Cpu {
        Cpu {
            pc: 0,
            regs: [0; 32],
            csr: [0; 4096],
            mem,
            mode: PrivMode::Machine,
        }
    }
}
