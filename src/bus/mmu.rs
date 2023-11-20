use crate::cpu::csr::*;
use crate::cpu::*;
use crate::util::read_bits;

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
}

impl Pte {
    pub fn default() -> Pte {
        Pte {
            pte: 0,
            phys_base: 0,
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
            AccessType::Load => Err(Exception::LoadAccessFault(addr)),
            AccessType::Store => Err(Exception::StoreAccessFault(addr)),
            AccessType::Fetch => Err(Exception::InstructionAccessFault(addr)),
        }
    }
}

pub struct Sv39Mmu {
    mppn: BusType,
}

impl Mmu for Sv39Mmu {
    type PnArr = [BusType; 2];

    fn new() -> Self {
        Sv39Mmu { mppn: 0 }
    }

    fn translate(&self, addr: BusType, access_type: AccessType) -> Result<BusType, Exception> {
        if self.mppn == 0 {
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

            if bus::get_bus()
                .store_nommu(addr, pte.pte, self.get_pte_size() * 8)
                .is_err()
            {
                return Self::create_exeption(addr, access_type);
            }
        }

        Ok(pte.phys_base | (addr & 0xfff))
    }

    fn get_pte(&self, addr: BusType, access_type: AccessType) -> Result<Pte, Exception> {
        let bus_instance = bus::get_bus();

        let levels = self.get_levels();
        let pte_size = self.get_pte_size();
        let vpn = self.get_vpn(addr, levels);

        let mut a = self.mppn;
        let mut i: i32 = (levels - 1) as i32;

        let mut pte = Pte::default();

        while i >= 0 {
            let pte_addr = a + vpn[i as usize] * pte_size;
            let _pte = bus_instance.load_nommu(pte_addr, pte_size * 8);

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
            let execute = PteBitTest!(pte.pte, PteBitVal::Execute);

            if !read && write {
                return Err(Self::create_exeption(addr, access_type).err().unwrap());
            }

            if read || execute {
                break;
            }

            a = (pte.pte >> 10) & 0x3fffff;

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

        for j in 0..i {
            pte.phys_base |= vpn[j as usize] << (12 + (j * 9));
        }

        for j in (i as u32)..levels {
            pte.phys_base |= ppn[j as usize] << (12 + (j * 9));
        }

        Ok(pte)
    }

    fn update(&mut self, satp: CsrType) {
        let ppn = read_bits(satp, 0, 22);

        self.mppn = (ppn << 12) as BusType;
    }

    fn get_levels(&self) -> BusType {
        return 2;
    }

    fn get_vpn(&self, addr: BusType, level: BusType) -> Self::PnArr {
        let mut ret: Self::PnArr = Self::PnArr::default();

        for i in 0..level {
            ret[i as usize] = (addr >> (12 + i * 9)) & 0x1ff;
        }

        ret
    }

    fn get_ppn(&self, pte: BusType, level: BusType) -> Self::PnArr {
        let mut ret: Self::PnArr = Self::PnArr::default();

        for i in 0..level {
            ret[i as usize] = (pte >> (10 + i * 9)) & 0x1ff;
        }

        ret
    }

    fn get_pte_size(&self) -> BusType {
        return 4;
    }

    fn is_active(&self) -> bool {
        self.mppn != 0
    }
}
