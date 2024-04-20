use crate::{
    bus,
    cpu::{self, CpuReg},
    frontend::exec_core::RV_PAGE_OFFSET_MASK,
    xmem::{PageAllocator, PageState},
};

pub struct GpfnState {
    pub addr: CpuReg,
    state: PageState,
}

impl GpfnState {
    pub fn new(addr: CpuReg, state: PageState) -> GpfnState {
        GpfnState { addr, state }
    }

    pub fn default() -> GpfnState {
        GpfnState {
            addr: 0,
            state: PageState::ReadWrite,
        }
    }

    pub fn set_state(&mut self, state: PageState) {
        PageAllocator::mark_page(self.addr as *mut u8, 1, state)
            .expect("Failed to mark fastmem page");

        self.state = state;
    }

    pub fn get_state(&self) -> PageState {
        self.state
    }
}

pub struct GpfnStateSet {
    gpfn_set: hashbrown::HashMap<CpuReg, GpfnState>,
}

impl GpfnStateSet {
    pub fn new() -> GpfnStateSet {
        GpfnStateSet {
            gpfn_set: hashbrown::HashMap::new(),
        }
    }

    pub fn add_gpfn(&mut self, gpfn: CpuReg) {
        assert!(gpfn & RV_PAGE_OFFSET_MASK as CpuReg == 0);

        let gpfn_state = GpfnState::new(gpfn, PageState::ReadWrite);

        self.gpfn_set.insert(gpfn, gpfn_state);
    }

    pub fn remove_gpfn(&mut self, gpfn: CpuReg) {
        assert!(gpfn & RV_PAGE_OFFSET_MASK as CpuReg == 0);

        self.gpfn_set.remove(&gpfn);
    }

    pub fn contains_gpfn(&self, gpfn: CpuReg) -> bool {
        self.gpfn_set.contains_key(&gpfn)
    }

    pub fn get_gpfn_state(&self, gpfn: CpuReg) -> Option<&GpfnState> {
        self.gpfn_set.get(&gpfn)
    }

    pub fn get_gpfn_state_mut(
        &mut self,
        gpfn: CpuReg,
        translate_method: bus::mmu::AccessType,
    ) -> Option<&mut GpfnState> {
        let cpu = cpu::get_cpu();
        let bus = bus::get_bus();

        let phys_gpfn = bus.translate(gpfn, &mut cpu.mmu, translate_method);

        let gpfn = if let Ok(phys_gpfn) = phys_gpfn {
            phys_gpfn
        } else {
            gpfn
        };

        self.gpfn_set.get_mut(&gpfn)
    }

    pub fn set_gpfn_state(&mut self, gpfn: CpuReg, state: PageState) {
        if let Some(gpfn_state) = self.gpfn_set.get_mut(&gpfn) {
            gpfn_state.set_state(state);
        }
    }
}
