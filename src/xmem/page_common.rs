use crate::{util, xmem::PageAllocator};

pub struct CodePage {
    ptr: *mut u8,
    npages: usize,
    offset: usize,
    state: PageState,
}

impl CodePage {
    pub fn new() -> Self {
        let ptr = PageAllocator::allocate_pages(32).unwrap();

        println!("Allocated code page at {:p}", ptr);

        CodePage {
            ptr,
            npages: 32,
            offset: 0,
            state: PageState::ReadWrite,
        }
    }

    pub fn push(&mut self, data: &[u8]) -> Result<(), AllocationError> {
        let page_size = PageAllocator::get_page_size();

        if self.offset + data.len() > self.npages * page_size {
            let npages = std::cmp::max(
                util::align_up(self.offset + data.len(), page_size) / page_size,
                self.npages * 2, // A logarithmic growth strategy may be better
            );

            let new_ptr = PageAllocator::realloc_pages(self.ptr, self.npages, npages).unwrap();

            self.ptr = new_ptr;
            self.npages = npages;
        }

        unsafe {
            std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(self.offset), data.len());
        }

        self.offset += data.len();

        Ok(())
    }

    pub fn mark_rw(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::ReadWrite {
            return Ok(());
        }

        let res = PageAllocator::mark_page(self.ptr, self.npages, PageState::ReadWrite);

        if res.is_ok() {
            self.state = PageState::ReadWrite;
        }

        res
    }

    pub fn mark_rx(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::ReadExecute {
            return Ok(());
        }

        let res = PageAllocator::mark_page(self.ptr, self.npages, PageState::ReadExecute);

        if res.is_ok() {
            self.state = PageState::ReadExecute;
        }

        res
    }

    pub fn mark_invalid(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::Invalid {
            return Ok(());
        }

        let res = PageAllocator::mark_page(self.ptr, self.npages, PageState::Invalid);

        if res.is_ok() {
            self.state = PageState::Invalid;
        }

        res
    }

    pub fn dealloc(&mut self) {
        PageAllocator::free_pages(self.ptr, self.npages);

        self.ptr = std::ptr::null_mut();
        self.npages = 0;
        self.offset = 0;
        self.state = PageState::Invalid;
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn as_end_ptr(&self) -> *mut u8 {
        unsafe { self.as_ptr().add(self.size()) }
    }

    pub fn size(&self) -> usize {
        self.offset
    }

    pub fn npages(&self) -> usize {
        self.npages
    }

    pub fn state(&self) -> PageState {
        self.state
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AllocationError {
    UnknownError,
    OutOfMemory,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PageState {
    ReadWrite,
    ReadExecute,
    Invalid,
}
