use crate::bus::BusType;
use hashbrown::HashMap;

use super::exec_core::{RV_PAGE_MASK, RV_PAGE_SIZE};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InsnMappingData {
    pub host_ptr: *mut u8,
    pub guest_idx: BusType,
    pub jit_block_idx: usize,
}

pub struct InsnData {
    mapping: HashMap<BusType, InsnMappingData>,
}

impl InsnData {
    pub fn new() -> InsnData {
        InsnData {
            mapping: HashMap::new(),
        }
    }

    pub fn add_mapping(&mut self, guest_idx: BusType, host_ptr: *mut u8, jit_block_idx: usize) {
        self.mapping.insert(
            guest_idx,
            InsnMappingData {
                host_ptr,
                guest_idx,
                jit_block_idx,
            },
        );
    }

    pub fn get_by_guest_idx(&self, guest_idx: BusType) -> Option<&InsnMappingData> {
        self.mapping.get(&guest_idx)
    }

    pub fn get_by_host_ptr(&self, host_ptr: *mut u8) -> Option<&InsnMappingData> {
        for (_, mapping) in self.mapping.iter() {
            if mapping.host_ptr == host_ptr {
                return Some(mapping);
            }
        }

        None
    }

    pub fn remove_by_guest_idx(&mut self, guest_idx: BusType) {
        self.mapping.remove(&guest_idx);
    }

    pub fn remove_by_guest_region(&mut self, guest_start: BusType, guest_end: BusType) {
        for i in guest_start..guest_end {
            self.remove_by_guest_idx(i);
        }
    }

    pub fn remove_by_guest_page(&mut self, guest_idx: BusType) {
        let page_start = guest_idx & RV_PAGE_MASK as BusType;
        let page_end = page_start + RV_PAGE_SIZE as BusType;

        self.remove_by_guest_region(page_start, page_end);
    }
}
