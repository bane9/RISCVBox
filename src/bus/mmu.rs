use crate::cpu::*;
use crate::frontend::exec_core::{RV_PAGE_OFFSET_MASK, RV_PAGE_SIZE};
use crate::{cpu::csr::*, util::read_bit};

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

    fn translate(&self, addr: BusType, access_type: AccessType) -> Result<BusType, Exception>;
    fn get_pte(&self, addr: BusType, access_type: AccessType) -> Result<Pte, Exception>;
    fn update(&mut self, satp: CsrType);
    fn get_levels(&self) -> BusType;
    fn get_pte_size(&self) -> BusType;
    fn is_active(&self) -> bool;

    type PnArr;

    fn get_vpn(&self, addr: BusType, level: BusType) -> Self::PnArr;
    fn get_ppn(&self, pte: BusType, level: BusType) -> Self::PnArr;

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

    fn translate(&self, addr: BusType, access_type: AccessType) -> Result<BusType, Exception> {
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

        let pte = self.get_pte(addr, access_type);

        if pte.is_err() {
            return Self::create_exeption(addr, access_type);
        }

        let mut pte = pte.unwrap();

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
        let dirty = PteBitTest!(pte.pte, PteBitVal::Dirty);

        if !accessed || (access_type == AccessType::Store && !dirty) {
            pte.pte |= PteBit!(PteBitVal::Accessed);

            if access_type == AccessType::Store {
                pte.pte |= PteBit!(PteBitVal::Dirty);
            }

            // TODO: make atomic
            if bus::get_bus()
                .store_nommu(pte.pte_addr, pte.pte, self.get_pte_size() * 8)
                .is_err()
            {
                return Self::create_exeption(addr, access_type);
            }
        }

        Ok(pte.phys_base | (addr & RV_PAGE_OFFSET_MASK as BusType))
    }

    fn get_pte(&self, addr: BusType, access_type: AccessType) -> Result<Pte, Exception> {
        let bus_instance = bus::get_bus();

        let levels = self.get_levels();
        let pte_size = self.get_pte_size();
        let vpn = self.get_vpn(addr, levels);

        let mut a = self.ppn;
        let mut i: i32 = (levels - 1) as i32;

        let mut pte = Pte::default();

        while i >= 0 {
            pte.pte_addr = a + vpn[i as usize] * pte_size;
            let _pte = bus_instance.load_nommu(pte.pte_addr, pte_size * 8);

            if _pte.is_err() {
                return Err(Self::create_exeption(addr, access_type).err().unwrap());
            }

            pte.pte = _pte.unwrap();

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
                pte.phys_base = ppn[0] << 12;
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
