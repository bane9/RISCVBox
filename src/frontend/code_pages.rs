use crate::backend::common::HostEncodedInsn;
use crate::xmem::{self, AllocationError, CodePage, PageState};

pub struct CodePages {
    xmem: Vec<xmem::CodePageImpl>,
}

impl CodePages {
    pub fn new() -> CodePages {
        CodePages { xmem: vec![] }
    }

    pub fn get_code_page(&mut self, idx: usize) -> &mut xmem::CodePageImpl {
        self.xmem.get_mut(idx).unwrap()
    }

    pub fn alloc_code_page(&mut self) -> (&mut xmem::CodePageImpl, usize) {
        self.xmem.push(xmem::CodePageImpl::new());
        let idx = self.xmem.len() - 1;
        (self.xmem.get_mut(idx).unwrap(), idx)
    }

    pub fn apply_insn(&mut self, idx: usize, insn: HostEncodedInsn) -> Result<(), AllocationError> {
        self.xmem[idx].push(insn.as_slice())
    }

    pub fn mark_all_pages(&mut self, state: PageState) {
        for xmem in self.xmem.iter_mut() {
            match state {
                PageState::ReadWrite => xmem.mark_rw().unwrap(),
                PageState::ReadExecute => xmem.mark_rx().unwrap(),
                PageState::Invalid => xmem.mark_invalid().unwrap(),
            }
        }
    }

    pub fn count(&self) -> usize {
        self.xmem.len()
    }
}
