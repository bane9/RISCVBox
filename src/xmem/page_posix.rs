use crate::xmem::page_common::{PageAllocator, AllocationError};
use std::ptr;
use libc::{mmap, mprotect, munmap, PROT_READ, PROT_WRITE, PROT_EXEC, MAP_ANON, MAP_PRIVATE};

const PAGE_SIZE: usize = 4096;

pub struct PosixAllocator;

fn posix_mark_page(ptr: *mut u8, npages: usize, protect: i32) -> Result<(), AllocationError> {
    let size = npages * PAGE_SIZE;
    let result = unsafe {
        mprotect(ptr as *mut _, size, protect)
    };

    if result != 0 {
        Err(AllocationError::UnknownError)
    } else {
        Ok(())
    }
}

impl PageAllocator for PosixAllocator {
    fn alloc(npages: usize) -> Result<*mut u8, AllocationError> {
        let size = npages * PAGE_SIZE;
        let addr = unsafe {
            mmap(ptr::null_mut(), size, PROT_READ | PROT_WRITE, MAP_ANON | MAP_PRIVATE, -1, 0)
        };

        if addr.is_null() {
            Err(AllocationError::OutOfMemory)
        } else {
            Ok(addr as *mut u8)
        }
    }

    fn realloc(old_ptr: *mut u8, old_npages: usize, new_npages: usize) -> Result<*mut u8, AllocationError> {
        if old_ptr.is_null() {
            return Self::alloc(new_npages);
        }
    
        let new_ptr = Self::alloc(new_npages)?;

        if new_ptr.is_null() {
            return Err(AllocationError::OutOfMemory);
        }
    
        let old_size = old_npages * PAGE_SIZE;
        let new_size = new_npages * PAGE_SIZE;
        let bytes_to_copy = std::cmp::min(old_size, new_size);
    
        unsafe {
            ptr::copy_nonoverlapping(old_ptr, new_ptr, bytes_to_copy);
        }
    
        Self::dealloc(old_ptr, old_size / PAGE_SIZE);
    
        Ok(new_ptr)
    }
    

    fn mark_rw(ptr: *mut u8, npages: usize) -> Result<(), AllocationError> {
        posix_mark_page(ptr, npages, PROT_READ | PROT_WRITE)
    }

    fn mark_rx(ptr: *mut u8, npages: usize) -> Result<(), AllocationError> {
        posix_mark_page(ptr, npages, PROT_READ | PROT_EXEC)
    }

    fn dealloc(ptr: *mut u8, npages: usize) {
        unsafe {
            munmap(ptr as *mut _, npages * PAGE_SIZE);
        }
    }

    fn page_size() -> usize {
        PAGE_SIZE
    }
}
