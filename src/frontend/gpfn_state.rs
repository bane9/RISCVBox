use std::collections::HashSet;

use crate::{cpu::CpuReg, frontend::exec_core::RV_PAGE_MASK};

pub struct GpfnState {
    gpfn_set: HashSet<CpuReg>,
}

impl GpfnState {
    pub fn new() -> GpfnState {
        GpfnState {
            gpfn_set: HashSet::new(),
        }
    }

    pub fn add_gpfn(&mut self, gpfn: CpuReg) {
        assert!(gpfn & RV_PAGE_MASK as CpuReg == 0);

        self.gpfn_set.insert(gpfn);
    }

    pub fn remove_gpfn(&mut self, gpfn: CpuReg) {
        assert!(gpfn & RV_PAGE_MASK as CpuReg == 0);

        self.gpfn_set.remove(&gpfn);
    }

    pub fn contains_gpfn(&self, gpfn: CpuReg) -> bool {
        return self.gpfn_set.contains(&gpfn);
    }
}
