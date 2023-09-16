use crate::xmem::page_common::{AllocationError, PageAllocator};

#[cfg(target_os = "windows")]
use crate::xmem::page_win32::Win32Allocator as XmemAllocator;

#[cfg(unix)]
use crate::xmem::page_posix::PosixAllocator as XmemAllocator;

pub struct Xmem {
    ptr: *mut u8,
    npages: usize,
}

impl Xmem {
    pub fn new_empty() -> Xmem {
        Xmem {
            ptr: std::ptr::null_mut(),
            npages: 0,
        }
    }

    pub fn new(initial_npages: usize) -> Result<Xmem, AllocationError> {
        let ptr = XmemAllocator::alloc(initial_npages)?;
        Ok(Xmem {
            ptr,
            npages: initial_npages,
        })
    }

    pub fn realloc(&mut self, new_npages: usize) -> Result<(), AllocationError> {
        let new_ptr = XmemAllocator::realloc(self.ptr, self.npages, new_npages)?;
        self.ptr = new_ptr;
        self.npages = new_npages;
        Ok(())
    }

    pub fn mark_rw(page: *mut u8) -> Result<(), AllocationError> {
        assert!(page as usize % XmemAllocator::page_size() == 0);
        XmemAllocator::mark_rw(page, 1)
    }

    pub fn mark_rx(page: *mut u8) -> Result<(), AllocationError> {
        assert!(page as usize % XmemAllocator::page_size() == 0);
        XmemAllocator::mark_rx(page, 1)
    }

    pub fn mark_invalid(page: *mut u8) -> Result<(), AllocationError> {
        assert!(page as usize % XmemAllocator::page_size() == 0);
        XmemAllocator::mark_rx(page, 1)
    }

    pub fn dealloc(&mut self) {
        XmemAllocator::dealloc(self.ptr, self.npages);
        self.ptr = std::ptr::null_mut();
        self.npages = 0;
    }

    pub fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    pub fn page_size() -> usize {
        XmemAllocator::page_size()
    }
}
