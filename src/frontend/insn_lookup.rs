use std::collections::HashMap;

use crate::bus::BusType;

pub struct InsnMappingData {
    pub host_ptr: *mut u8,
    pub guest_idx: BusType,
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

    pub fn add_mapping(&mut self, guest_idx: BusType, host_ptr: *mut u8) {
        self.mapping.insert(
            guest_idx,
            InsnMappingData {
                host_ptr,
                guest_idx,
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
}
