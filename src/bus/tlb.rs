use crate::bus::bus::*;
use crate::frontend::exec_core::RV_PAGE_SHIFT;

const TLB_ENTRIES: usize = 16;
const MAX_ASID_ENTRIES: usize = 16;

#[derive(Debug, Clone, Copy)]
pub struct TlbEntry {
    pub virt: BusType,
    pub phys: BusType,
}

pub struct TLBAsidEntry {
    tlb: [TlbEntry; TLB_ENTRIES],
}

impl TLBAsidEntry {
    pub fn new() -> TLBAsidEntry {
        TLBAsidEntry {
            tlb: [TlbEntry { virt: 0, phys: 0 }; TLB_ENTRIES],
        }
    }

    #[inline]
    pub fn get_ppn_entry(&self, virt: BusType) -> BusType {
        let vpn = (virt >> RV_PAGE_SHIFT) as BusType;
        
        let tlb_entry = &self.tlb[vpn as usize % TLB_ENTRIES];

        if tlb_entry.virt == vpn {
            return tlb_entry.phys;
        }

        0
    }

    #[inline]
    pub fn set_ppn_entry(&mut self, virt: BusType, phys: BusType) {
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

#[thread_local]
static mut TLB: *mut TLBAsidEntry = std::ptr::null_mut();

pub fn tlb_init() {
    unsafe {
        TLB = Box::into_raw(Box::new(TLBAsidEntry::new()));
    }
}

pub fn get_current_tlb() -> &'static mut TLBAsidEntry {
    unsafe {
        &mut *TLB
    }
}

