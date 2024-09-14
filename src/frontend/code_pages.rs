use hashbrown::HashMap;

use crate::backend::common::HostEncodedInsn;
use crate::xmem::{self, AllocationError, PageState};

pub struct CodePages {
    xmem: HashMap<usize, xmem::CodePage>,
    idx: usize,
}

impl CodePages {
    pub fn new() -> CodePages {
        CodePages {
            xmem: HashMap::new(),
            idx: 0,
        }
    }

    pub fn get_code_page(&mut self, idx: usize) -> &mut xmem::CodePage {
        self.xmem.get_mut(&idx).unwrap()
    }

    pub fn alloc_code_page(&mut self) -> (&mut xmem::CodePage, usize) {
        let idx = self.idx;
        self.idx += 1;

        let xmem = xmem::CodePage::new();
        self.xmem.insert(idx, xmem);

        (self.xmem.get_mut(&idx).unwrap(), idx)
    }

    pub fn apply_insn(&mut self, idx: usize, insn: HostEncodedInsn) -> Result<(), AllocationError> {
        self.xmem.get_mut(&idx).unwrap().push(insn.as_slice())
    }

    pub fn remove_code_page(&mut self, idx: usize) {
        self.xmem.get_mut(&idx).unwrap().dealloc();
        self.xmem.remove(&idx);
    }

    pub fn mark_all_pages(&mut self, state: PageState) {
        for xmem in self.xmem.iter_mut() {
            match state {
                PageState::ReadWrite => xmem.1.mark_rw().unwrap(),
                PageState::ReadExecute => xmem.1.mark_rx().unwrap(),
                PageState::Invalid => xmem.1.mark_invalid().unwrap(),
            }
        }
    }

    pub fn count(&self) -> usize {
        self.xmem.len()
    }

    pub fn cleanup(&mut self) {
        for xmem in self.xmem.iter_mut() {
            xmem.1.dealloc();
        }

        self.xmem.clear();
    }
}
