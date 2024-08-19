use crate::cpu::*;
use crate::frontend::exec_core::{RV_PAGE_OFFSET_MASK, RV_PAGE_SIZE};
use crate::util::read_bits;
use crate::{cpu::csr::*, util::read_bit};

use super::tlb::{asid_tlb_set, get_current_tlb};
use super::{bus, BusType};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AccessType {
    Load,
    Store,
    Fetch,
}

pub enum PteBitVal {
    Valid = 0,
    Read = 1,
    Write = 2,
    Execute = 3,
    User = 4,
    Global = 5,
    Accessed = 6,
    Dirty = 7,
}

#[derive(Debug, Clone, Copy)]
pub struct Pte {
    pte: BusType,
    phys_base: BusType,
    pte_addr: BusType,
}

impl Pte {
    pub fn default() -> Pte {
        Pte {
            pte: 0,
            phys_base: 0,
            pte_addr: 0,
        }
    }
}

macro_rules! PteBit {
    ($pte_bit: expr) => {
        (1 << ($pte_bit as usize))
    };
}

macro_rules! PteBitTest {
    ($val: expr, $pte_bit: expr) => {
        ($val & PteBit!($pte_bit)) != 0
    };
}

pub trait Mmu {
    fn new() -> Self;

    fn get_pte(&mut self, addr: BusType, access_type: AccessType) -> Result<Pte, Exception>;
    fn update(&mut self, satp: CsrType);
    fn get_levels(&self) -> BusType;
    fn get_pte_size(&self) -> BusType;
    fn is_active(&self) -> bool;

    type PnArr;

    fn get_vpn(&self, addr: BusType, level: BusType) -> Self::PnArr;
    fn get_ppn(&self, pte: BusType, level: BusType) -> Self::PnArr;

    fn translate(&mut self, addr: BusType, access_type: AccessType) -> Result<BusType, Exception> {
        if !self.is_active() {
            return Ok(addr);
        }

        let cpu_instance = cpu::get_cpu();

        let mut mode = cpu_instance.mode;

        if access_type != AccessType::Fetch && cpu_instance.csr.read_bit_mstatus(csr::bits::MPRV) {
            mode = cpu_instance.csr.read_mpp_mode();
        }

        if mode == MppMode::Machine {
            return Ok(addr);
        }

        let mut pte = {
            let pte = self.get_pte(addr, access_type);

            if pte.is_err() {
                return Self::create_exeption(addr, access_type);
            }

            pte.unwrap()
        };

        let mxr = cpu_instance.csr.read_bit_mstatus(csr::bits::MXR);
        let sum = cpu_instance.csr.read_bit_mstatus(csr::bits::SUM);

        let read = PteBitTest!(pte.pte, PteBitVal::Read);
        let write = PteBitTest!(pte.pte, PteBitVal::Write);
        let execute = PteBitTest!(pte.pte, PteBitVal::Execute);

        if (!read && write && !execute) || (!read && write && execute) {
            return Self::create_exeption(addr, access_type);
        }

        let user = PteBitTest!(pte.pte, PteBitVal::User);

        if user && ((mode != MppMode::User) && (!sum || access_type == AccessType::Fetch)) {
            return Self::create_exeption(addr, access_type);
        }

        if !user && mode != MppMode::Supervisor {
            return Self::create_exeption(addr, access_type);
        }

        match access_type {
            AccessType::Load => {
                if !(read || (execute && mxr)) {
                    return Self::create_exeption(addr, access_type);
                }
            }
            AccessType::Store => {
                if !write {
                    return Self::create_exeption(addr, access_type);
                }
            }
            AccessType::Fetch => {
                if !execute {
                    return Self::create_exeption(addr, access_type);
                }
            }
        }

        let accessed = PteBitTest!(pte.pte, PteBitVal::Accessed);
        let mut dirty = PteBitTest!(pte.pte, PteBitVal::Dirty);

        if !accessed || (access_type == AccessType::Store && !dirty) {
            pte.pte |= PteBit!(PteBitVal::Accessed);

            if access_type == AccessType::Store {
                pte.pte |= PteBit!(PteBitVal::Dirty);
                dirty = true;
            }

            let pte_atomic: &std::sync::atomic::AtomicU32 =
                unsafe { std::mem::transmute(pte.pte_addr as u64) };

            pte_atomic.store(pte.pte, std::sync::atomic::Ordering::Release);
        }

        let phys_flags = if write && dirty {
            pte.phys_base | 0x1
        } else {
            pte.phys_base
        };

        get_current_tlb().set_phys_entry(addr, phys_flags);

        Ok(pte.phys_base | (addr & RV_PAGE_OFFSET_MASK as BusType))
    }

    fn create_exeption(addr: BusType, access_type: AccessType) -> Result<BusType, Exception> {
        match access_type {
            AccessType::Load => Err(Exception::LoadPageFault(addr)),
            AccessType::Store => Err(Exception::StorePageFault(addr)),
            AccessType::Fetch => Err(Exception::InstructionPageFault(addr)),
        }
    }
}

pub struct Sv32Mmu {
    ppn: BusType,
    enabled: bool,
}

impl Mmu for Sv32Mmu {
    type PnArr = [BusType; 2];

    fn new() -> Self {
        Sv32Mmu {
            ppn: 0,
            enabled: false,
        }
    }

    fn get_pte(&mut self, addr: BusType, access_type: AccessType) -> Result<Pte, Exception> {
        let levels = self.get_levels();
        let pte_size = self.get_pte_size();
        let vpn = self.get_vpn(addr, levels);

        let mut a = self.ppn;
        let mut i: i32 = (levels - 1) as i32;

        let mut pte = Pte::default();
        let bus = bus::get_bus();

        while i >= 0 {
            pte.pte_addr = a + vpn[i as usize] * pte_size;

            if !bus.is_dram_addr(pte.pte_addr) {
                return Err(Self::create_exeption(addr, access_type).err().unwrap());
            }

            let pte_atomic: &std::sync::atomic::AtomicU32 =
                unsafe { std::mem::transmute(pte.pte_addr as u64) };

            pte.pte = pte_atomic.load(std::sync::atomic::Ordering::Acquire);

            let valid = PteBitTest!(pte.pte, PteBitVal::Valid);

            if !valid {
                return Err(Self::create_exeption(addr, access_type).err().unwrap());
            }

            let read = PteBitTest!(pte.pte, PteBitVal::Read);
            let write = PteBitTest!(pte.pte, PteBitVal::Write);

            if !read && write {
                return Err(Self::create_exeption(addr, access_type).err().unwrap());
            }

            let execute = PteBitTest!(pte.pte, PteBitVal::Execute);

            if read || execute {
                break;
            }

            a = ((pte.pte >> 10) & 0x3fffff) * RV_PAGE_SIZE as CpuReg;

            i -= 1;
        }

        if i < 0 {
            return Err(Self::create_exeption(addr, access_type).err().unwrap());
        }

        let ppn = self.get_ppn(pte.pte, levels);

        for j in 0..i {
            if ppn[j as usize] != 0 {
                return Err(Self::create_exeption(addr, access_type).err().unwrap());
            }
        }

        match i {
            0 => {
                pte.phys_base = (ppn[1] << 22) | (ppn[0] << 12);
            }
            1 => {
                pte.phys_base = (ppn[1] << 22) | (vpn[0] << 12);
            }
            _ => {
                unreachable!()
            }
        }

        Ok(pte)
    }

    fn update(&mut self, satp: CsrType) {
        self.ppn = (satp & 0x3fffff) * RV_PAGE_SIZE as CpuReg;

        self.enabled = read_bit(satp, 31);

        let asid = read_bits(satp, 30, 22);

        asid_tlb_set(asid as usize);
        get_current_tlb().flush();
    }

    fn get_levels(&self) -> BusType {
        2
    }

    fn get_vpn(&self, addr: BusType, _level: BusType) -> Self::PnArr {
        let mut ret = Self::PnArr::default();

        ret[0] = (addr >> 12) & 0x3ff;
        ret[1] = (addr >> 22) & 0x3ff;

        ret
    }

    fn get_ppn(&self, pte: BusType, _level: BusType) -> Self::PnArr {
        let mut ret = Self::PnArr::default();

        ret[0] = (pte >> 10) & 0x3ff;
        ret[1] = (pte >> 20) & 0xfff;

        ret
    }

    fn get_pte_size(&self) -> BusType {
        4
    }

    fn is_active(&self) -> bool {
        self.enabled
    }
}
