use crate::bus::bus::*;
use crate::frontend::exec_core::RV_PAGE_SHIFT;

const TLB_ENTRIES: usize = 32;
const MAX_ASID_ENTRIES: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct TlbEntry {
    pub virt: BusType,
    pub phys: BusType,
}

pub struct TLBAsidEntry {
    tlb: [TlbEntry; TLB_ENTRIES],
}

#[macro_export]
macro_rules! tlb_fetch_load {
    ($addr:expr) => {
        let phys = get_current_tlb().get_phys_entry($addr as CpuReg) as usize;

        if phys != 0 {
            let phys = phys & !1;

            return (phys | ($addr & RV_PAGE_OFFSET_MASK)) as usize;
        }
    };
}

#[macro_export]
macro_rules! tlb_fetch_store {
    ($addr:expr) => {
        let phys = get_current_tlb().get_phys_entry($addr as CpuReg) as usize;

        let is_write = (phys & 1) != 0;
        if phys != 0 && is_write {
            let phys = phys & !1;

            return (phys | ($addr & RV_PAGE_OFFSET_MASK)) as usize;
        }
    };
}

impl TLBAsidEntry {
    pub fn new() -> TLBAsidEntry {
        TLBAsidEntry {
            tlb: [TlbEntry { virt: 0, phys: 0 }; TLB_ENTRIES],
        }
    }

    #[inline]
    pub fn get_phys_entry(&self, virt: BusType) -> BusType {
        let vpn = (virt >> RV_PAGE_SHIFT) as BusType;

        let tlb_entry = &self.tlb[vpn as usize % TLB_ENTRIES];

        if tlb_entry.virt == vpn {
            return tlb_entry.phys;
        }

        0
    }

    #[inline]
    pub fn set_phys_entry(&mut self, virt: BusType, phys: BusType) {
        let virt = (virt >> RV_PAGE_SHIFT) as BusType;

        self.tlb[virt as usize % TLB_ENTRIES] = TlbEntry { virt, phys };
    }

    #[inline]
    pub fn flush(&mut self) {
        for i in 0..TLB_ENTRIES {
            self.tlb[i] = TlbEntry { virt: 0, phys: 0 };
        }
    }
}

static mut TLB: *mut TLBAsidEntry = std::ptr::null_mut();

pub fn get_current_tlb() -> &'static mut TLBAsidEntry {
    unsafe { &mut *TLB }
}

pub struct AsidAllocator {
    asid: [usize; MAX_ASID_ENTRIES],
    lru: [usize; MAX_ASID_ENTRIES],
    counter: usize,
}

impl AsidAllocator {
    pub fn new() -> AsidAllocator {
        AsidAllocator {
            asid: [0; MAX_ASID_ENTRIES],
            lru: [0; MAX_ASID_ENTRIES],
            counter: 0,
        }
    }

    pub fn set_asid(&mut self, asid: usize) {
        let mut empty_index: Option<usize> = None;
        let mut lru_index = 0;

        for i in 0..MAX_ASID_ENTRIES {
            if self.asid[i] == asid {
                self.lru[i] = self.counter;
                self.counter += 1;
                return;
            }

            if self.asid[i] == 0 && empty_index.is_none() {
                empty_index = Some(i);
            }

            if self.lru[i] < self.lru[lru_index] {
                lru_index = i;
            }
        }

        let target_index = if let Some(index) = empty_index {
            index
        } else {
            lru_index
        };

        self.asid[target_index] = asid;
        self.lru[target_index] = self.counter;
        self.counter += 1;
        unsafe {
            TLB = &self.asid[target_index] as *const usize as *mut TLBAsidEntry;
        }
    }
}

static mut ASID_ALLOCATOR: *mut AsidAllocator = std::ptr::null_mut();

pub fn asid_tlb_init() {
    unsafe {
        ASID_ALLOCATOR = Box::into_raw(Box::new(AsidAllocator::new()));
        TLB = &(*ASID_ALLOCATOR).asid[0] as *const usize as *mut TLBAsidEntry;
    }
}

pub fn asid_tlb_set(asid: usize) {
    unsafe {
        (*ASID_ALLOCATOR).set_asid(asid);
    }
}
