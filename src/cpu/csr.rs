use chrono::Utc;

use crate::bus::bus::BusType;
use crate::util::util;
use std::cell::RefCell;

pub const CSR_COUNT: usize = 4096;
pub type CsrType = BusType;

// Constants

pub mod register {
    pub const USTATUS: usize = 0x0;
    pub const FFLAGS: usize = 0x1;
    pub const FRM: usize = 0x2;
    pub const FCSR: usize = 0x3;
    pub const UVEC: usize = 0x5;
    pub const UEPC: usize = 0x41;
    pub const UCAUSE: usize = 0x42;
    pub const UTVAL: usize = 0x43;
    pub const SSTATUS: usize = 0x100;
    pub const SEDELEG: usize = 0x102;
    pub const SIDELEG: usize = 0x103;
    pub const SIE: usize = 0x104;
    pub const STVEC: usize = 0x105;
    pub const SSCRATCH: usize = 0x140;
    pub const SEPC: usize = 0x141;
    pub const SCAUSE: usize = 0x142;
    pub const STVAL: usize = 0x143;
    pub const SIP: usize = 0x144;
    pub const SATP: usize = 0x180;
    pub const MSTATUS: usize = 0x300;
    pub const MISA: usize = 0x301;
    pub const MEDELEG: usize = 0x302;
    pub const MIDELEG: usize = 0x303;
    pub const MIE: usize = 0x304;
    pub const MTVEC: usize = 0x305;
    pub const MCOUNTEREN: usize = 0x306;
    pub const MSCRATCH: usize = 0x340;
    pub const MEPC: usize = 0x341;
    pub const MCAUSE: usize = 0x342;
    pub const MTVAL: usize = 0x343;
    pub const MIP: usize = 0x344;
    pub const CYCLE: usize = 0xc00;
    pub const TIME: usize = 0xc01;
    pub const TIMEMS: usize = 0xc10;
    pub const TDATA1: usize = 0x7a1;
    pub const MVENDORID: usize = 0xf11;
    pub const MARCHID: usize = 0xf12;
    pub const MIMPID: usize = 0xf13;
    pub const MHARTID: usize = 0xf14;
}

pub const SIE: usize = 1 << 1;
pub const SPIE: usize = 1 << 5;
pub const UBE: usize = 1 << 6;
pub const SPP: usize = 1 << 8;
pub const FS: usize = 0x6000;
pub const XS: usize = 0x18000;
pub const SUM: usize = 1 << 18;
pub const MXR: usize = 1 << 19;

pub const SSTATUS: usize = SIE | SPIE | UBE | SPP | FS | XS | SUM | MXR;

pub const MIE: usize = 1 << 3;
pub const MPIE: usize = 1 << 7;
pub const MPP: usize = 1 << 12;
pub const MPRV: usize = 1 << 17;
pub const TVM: usize = 1 << 20;
pub const TSR: usize = 1 << 22;

pub const A_EXT: usize = 1 << 0;
pub const C_EXT: usize = 1 << 2;
pub const D_EXT: usize = 1 << 3;
pub const RV32E: usize = 1 << 4;
pub const F_EXT: usize = 1 << 5;
pub const HYPERVISOR: usize = 1 << 7;
pub const RV32I_64I_128I: usize = 1 << 8;
pub const M_EXT: usize = 1 << 12;
pub const N_EXT: usize = 1 << 13;
pub const QUAD_EXT: usize = 1 << 16;
pub const SUPERVISOR: usize = 1 << 18;
pub const USER: usize = 1 << 20;
pub const NON_STD_PRESENT: usize = 1 << 22;

pub const XLEN_32: usize = 1 << 30;
pub const XLEN_64: usize = 2 << 62;

pub mod bits {
    pub const SIE: usize = 1;
    pub const SPIE: usize = 5;
    pub const SPP: usize = 8;
    pub const MIE: usize = 3;
    pub const MPIE: usize = 7;
    pub const MPP: usize = 12;
    pub const MPRV: usize = 17;
    pub const SUM: usize = 18;
    pub const MXR: usize = 19;
    pub const TVM: usize = 20;
    pub const TSR: usize = 22;
    pub const SSIP_BIT: usize = 1;
    pub const MSIP_BIT: usize = 3;
    pub const STIP_BIT: usize = 5;
    pub const MTIP_BIT: usize = 7;
    pub const SEIP_BIT: usize = 9;
    pub const MEIP_BIT: usize = 11;

    pub const SSIP: usize = 1 << SSIP_BIT;
    pub const MSIP: usize = 1 << MSIP_BIT;
    pub const STIP: usize = 1 << STIP_BIT;
    pub const MTIP: usize = 1 << MTIP_BIT;
    pub const SEIP: usize = 1 << SEIP_BIT;
    pub const MEIP: usize = 1 << MEIP_BIT;
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum MppMode {
    Machine = 3,
    Supervisor = 1,
    User = 0,
}

// Implementation

pub struct Csr {
    pub regs: [CsrType; CSR_COUNT],
}

impl Csr {
    pub fn new() -> Self {
        let mut regs = [0 as CsrType; CSR_COUNT];

        regs[register::MISA] =
            (XLEN_32 | RV32I_64I_128I | A_EXT | M_EXT | SUPERVISOR | USER) as u32;

        let csr = Self { regs };

        csr
    }

    pub fn read(&self, addr: usize) -> CsrType {
        match addr {
            register::SSTATUS => self.regs[register::MSTATUS] & SSTATUS as CsrType,
            register::SIE => self.regs[register::SIE] & self.regs[register::MIDELEG],
            register::SIP => self.regs[register::MIP] & self.regs[register::MIDELEG],
            register::CYCLE => Utc::now().timestamp_millis() as CsrType, // TODO: make it make sense v2
            _ => self.regs[addr],
        }
    }

    pub fn write(&mut self, addr: usize, data: CsrType) {
        match addr {
            register::SSTATUS => {
                let val = (self.regs[register::MSTATUS as usize] & !SSTATUS as u32)
                    | (data & SSTATUS as u32);
                self.regs[register::MSTATUS as usize] = val;
            }
            register::SIE => {
                let val = (self.regs[register::MIE] & !self.regs[register::MIDELEG as usize])
                    | (data & self.regs[register::MIDELEG as usize]);
                self.regs[register::MIE as usize] = val;
            }
            register::SIP => {
                let mask = self.regs[register::MIDELEG as usize] & bits::SSIP as u32;
                let val = (self.regs[register::MIP as usize] & !mask) | (data & mask);
                self.regs[register::MIP as usize] = val;
            }
            _ => {
                self.regs[addr as usize] = data;
            }
        }
    }

    pub fn read_bit(&self, addr: usize, bit: usize) -> bool {
        let val = self.read(addr);
        util::read_bit(val, bit)
    }

    pub fn write_bit(&mut self, addr: usize, bit: usize, bit_value: bool) {
        let val = self.read(addr);
        let val = util::write_bit(val as usize, bit, bit_value);
        self.write(addr, val as CsrType);
    }

    pub fn read_bits(&self, addr: usize, start: usize, end: usize) -> CsrType {
        let val = self.read(addr);
        util::read_bits(val, start, end) as u32
    }

    pub fn write_bits(&mut self, addr: usize, start: usize, end: usize, bits: CsrType) {
        let val = self.read(addr);
        let val = util::write_bits(val as usize, start, end, bits as usize);
        self.write(addr, val as CsrType);
    }

    pub fn read_bit_mstatus(&self, bit: usize) -> bool {
        self.read_bit(register::MSTATUS, bit)
    }

    pub fn write_bit_mstatus(&mut self, bit: usize, bit_value: bool) {
        self.write_bit(register::MSTATUS, bit, bit_value);
    }

    pub fn write_mpp_mode(&mut self, mode: MppMode) {
        self.write_bits(register::MSTATUS, 11, 12, mode as CsrType);
    }

    pub fn read_mpp_mode(&self) -> MppMode {
        let val = self.read_bits(register::MSTATUS, 11, 12);
        match val {
            0 => MppMode::User,
            1 => MppMode::Supervisor,
            3 => MppMode::Machine,
            _ => panic!("Invalid MPP mode"),
        }
    }

    pub fn read_bit_sstatus(&self, bit: usize) -> bool {
        self.read_bit(register::SSTATUS, bit)
    }

    pub fn write_bit_sstatus(&mut self, bit: usize, bit_value: bool) {
        self.write_bit(register::SSTATUS, bit, bit_value);
    }
}

thread_local! {
    static CSR: RefCell<Csr> = RefCell::new(Csr::new());
}

pub fn get_csr() -> &'static mut Csr {
    CSR.with(|csr| unsafe { &mut *csr.as_ptr() })
}
