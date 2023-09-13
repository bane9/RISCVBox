use crate::xmem::page_common::{ PageAllocator, AllocationError };

#[cfg(target_os = "windows")]
use crate::xmem::page_win32::Win32Allocator as XmemAllocator;

#[cfg(unix)]
use crate::xmem::page_posix::PosixAllocator as XmemAllocator;

pub struct Xmem {
    ptr: *mut u8,
    npages: usize,
}

impl Xmem {
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

    pub fn mark_rw(&mut self) -> Result<(), AllocationError> {
        XmemAllocator::mark_rw(self.ptr, self.npages)
    }

    pub fn mark_rx(&mut self) -> Result<(), AllocationError> {
        XmemAllocator::mark_rx(self.ptr, self.npages)
    }

    pub fn dealloc(&mut self) {
        XmemAllocator::dealloc(self.ptr, self.npages);
    }
}
