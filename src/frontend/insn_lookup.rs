use std::{collections::HashMap, sync::Arc};

use crate::bus::BusType;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct InsnMappingData {
    pub host_ptr: *mut u8,
    pub guest_idx: BusType,
    pub jit_block_idx: usize,
}

pub struct InsnData {
    mapping: HashMap<BusType, Arc<InsnMappingData>>,
}

impl InsnData {
    pub fn new() -> InsnData {
        InsnData {
            mapping: HashMap::new(),
        }
    }

    pub fn add_mapping(
        &mut self,
        guest_idx: BusType,
        host_ptr: *mut u8,
        jit_block_idx: usize,
        virt_addr: Option<BusType>,
    ) {
        if virt_addr.is_some() {
            let virt_addr = virt_addr.unwrap();

            let mapping = self.mapping.get(&guest_idx);

            if mapping.is_some() {
                self.mapping.insert(virt_addr, mapping.unwrap().clone());
            } else {
                let new = self
                    .mapping
                    .insert(
                        guest_idx,
                        Arc::new(InsnMappingData {
                            host_ptr,
                            guest_idx,
                            jit_block_idx,
                        }),
                    )
                    .unwrap();

                self.mapping.insert(virt_addr, new);
            }
        } else {
            self.mapping.insert(
                guest_idx,
                Arc::new(InsnMappingData {
                    host_ptr,
                    guest_idx,
                    jit_block_idx,
                }),
            );
        }
    }

    pub fn get_by_guest_idx(&self, guest_idx: BusType) -> Option<&InsnMappingData> {
        self.mapping.get(&guest_idx).map(|x| x.as_ref())
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
}
