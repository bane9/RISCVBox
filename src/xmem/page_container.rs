use crate::xmem::page_common::{AllocationError, PageAllocator};

#[cfg(target_os = "windows")]
use crate::xmem::page_win32::Win32Allocator as XmemAllocator;

#[cfg(unix)]
use crate::xmem::page_posix::PosixAllocator as XmemAllocator;

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum PageState {
    Invalid,
    ReadWrite,
    ReadExecute,
}

#[derive(Clone)]
pub struct Xmem {
    ptr: *mut u8,
    npages: usize,
    pub non_reserved_bytes: usize,
    pub used_bytes: usize,
    page_state: PageState,
}

impl Xmem {
    pub fn new_as_list(
        pages_total: usize,
        xmem_per_page: usize,
    ) -> Result<Vec<Xmem>, AllocationError> {
        assert!(
            pages_total > 0
                && xmem_per_page > 0
                && pages_total % xmem_per_page == 0
                && pages_total >= xmem_per_page
        );

        let mut xmem_list: Vec<Xmem> = Vec::new();

        let ptr = XmemAllocator::alloc(pages_total)?;

        for i in 0..pages_total / xmem_per_page {
            let xmem = Xmem {
                ptr: unsafe { ptr.add(i * xmem_per_page * XmemAllocator::page_size()) },
                npages: xmem_per_page,
                non_reserved_bytes: xmem_per_page * XmemAllocator::page_size(),
                used_bytes: 0,
                page_state: PageState::ReadWrite,
            };
            xmem_list.push(xmem);
        }

        Ok(xmem_list)
    }

    pub fn new(initial_npages: usize) -> Result<Xmem, AllocationError> {
        let ptr = XmemAllocator::alloc(initial_npages)?;
        Ok(Xmem {
            ptr,
            npages: initial_npages,
            non_reserved_bytes: initial_npages * XmemAllocator::page_size(),
            used_bytes: 0,
            page_state: PageState::ReadWrite,
        })
    }

    pub fn realloc(&mut self, new_npages: usize) -> Result<(), AllocationError> {
        let new_ptr = XmemAllocator::realloc(self.ptr, self.npages, new_npages)?;
        self.ptr = new_ptr;
        self.npages = new_npages;
        Ok(())
    }

    pub fn mark_rw(&mut self) -> Result<(), AllocationError> {
        XmemAllocator::mark_rw(self.ptr, self.npages)?;
        self.page_state = PageState::ReadWrite;
        Ok(())
    }

    pub fn mark_rx(&mut self) -> Result<(), AllocationError> {
        XmemAllocator::mark_rx(self.ptr, self.npages)?;
        self.page_state = PageState::ReadExecute;
        Ok(())
    }

    pub fn mark_invalid(&mut self) -> Result<(), AllocationError> {
        XmemAllocator::mark_invalid(self.ptr, self.npages)?;
        self.page_state = PageState::Invalid;
        Ok(())
    }

    pub fn get_state(&self) -> PageState {
        self.page_state
    }

    pub fn dealloc(&mut self) {
        XmemAllocator::dealloc(self.ptr, self.npages);
        self.ptr = std::ptr::null_mut();
        self.npages = 0;
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn get_npages(&self) -> usize {
        self.npages
    }

    pub fn get_size(&self) -> usize {
        self.npages * XmemAllocator::page_size()
    }

    pub fn end(&self) -> *mut u8 {
        unsafe { self.ptr.add(self.npages * XmemAllocator::page_size()) }
    }

    pub fn page_size() -> usize {
        XmemAllocator::page_size()
    }
}
