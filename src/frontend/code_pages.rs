use crate::backend::common::HostEncodedInsn;
use crate::xmem::page_container::{PageState, Xmem};

pub struct CodePages {
    pub xmem: Vec<Xmem>,
    pub pages_total: usize,
}

impl CodePages {
    pub fn new(pages_total: usize, pages_per_xmem: usize) -> CodePages {
        let xmem = Xmem::new_as_list(pages_total, pages_per_xmem).unwrap();
        CodePages { xmem, pages_total }
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.xmem[0].as_ptr()
    }

    pub fn get_xmem_from_page(&self, page: usize) -> Option<Xmem> {
        if page >= self.pages_total {
            return None;
        }

        Some(self.xmem[page].clone())
    }

    pub fn get_xmem_from_ptr(&self, ptr: *mut u8) -> Option<Xmem> {
        let first_page = self.xmem[0].as_ptr() as usize;
        let page: usize = (ptr as usize - first_page) / Xmem::page_size();

        if page >= self.pages_total {
            return None;
        }

        Some(self.xmem[page].clone())
    }

    pub fn apply_insn(&mut self, ptr: *mut u8, insn: HostEncodedInsn) -> Option<*mut u8> {
        let xmem = &mut self.get_xmem_from_ptr(ptr).unwrap();

        if insn.size() > xmem.non_reserved_bytes {
            return None;
        }

        let new_ptr = unsafe { ptr.add(insn.size()) };

        xmem.used_bytes += insn.size();

        xmem.mark_rw().unwrap();

        unsafe {
            std::ptr::copy_nonoverlapping(insn.as_ptr(), ptr, insn.size());
        }

        Some(new_ptr)
    }

    pub fn apply_reserved_insn(&mut self, ptr: *mut u8, insn: HostEncodedInsn) {
        let xmem = &mut self.get_xmem_from_ptr(ptr).unwrap();

        assert!(xmem.non_reserved_bytes == xmem.get_size());

        let new_ptr = xmem.end().wrapping_sub(insn.size());

        xmem.mark_rw().unwrap();

        unsafe {
            std::ptr::copy_nonoverlapping(new_ptr, ptr, insn.size());
        }

        xmem.non_reserved_bytes -= insn.size();
    }

    pub fn apply_reserved_insn_all(&mut self, insn: HostEncodedInsn) {
        for xmem in self.xmem.iter_mut() {
            assert!(xmem.non_reserved_bytes == xmem.get_size());

            let new_ptr = xmem.end().wrapping_sub(insn.size());

            xmem.mark_rw().unwrap();

            unsafe {
                std::ptr::copy_nonoverlapping(insn.as_ptr(), new_ptr, insn.size());
            }

            xmem.non_reserved_bytes -= insn.size();
        }
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
}
