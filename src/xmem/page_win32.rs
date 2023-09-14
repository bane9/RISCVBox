use crate::xmem::page_common::{AllocationError, PageAllocator};

extern crate winapi;

use std::ptr;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree, VirtualProtect};
use winapi::um::winnt::{MEM_COMMIT, MEM_RELEASE, PAGE_EXECUTE_READ, PAGE_READWRITE};

const PAGE_SIZE: usize = 4096;

pub struct Win32Allocator;

fn win32_mark_page(ptr: *mut u8, npages: usize, protect: u32) -> Result<(), AllocationError> {
    let size = npages * PAGE_SIZE;
    let mut old_protect = 0;
    let result = unsafe { VirtualProtect(ptr as *mut _, size, protect, &mut old_protect) };

    if result == 0 {
        Err(AllocationError::UnknownError)
    } else {
        Ok(())
    }
}

impl PageAllocator for Win32Allocator {
    fn alloc(npages: usize) -> Result<*mut u8, AllocationError> {
        let size = npages * PAGE_SIZE;
        let addr = unsafe { VirtualAlloc(ptr::null_mut(), size, MEM_COMMIT, PAGE_READWRITE) };

        if addr.is_null() {
            Err(AllocationError::OutOfMemory)
        } else {
            Ok(addr as *mut u8)
        }
    }

    fn realloc(
        old_ptr: *mut u8,
        old_npages: usize,
        new_npages: usize,
    ) -> Result<*mut u8, AllocationError> {
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
        win32_mark_page(ptr, npages, PAGE_READWRITE)
    }

    fn mark_rx(ptr: *mut u8, npages: usize) -> Result<(), AllocationError> {
        win32_mark_page(ptr, npages, PAGE_EXECUTE_READ)
    }

    fn dealloc(ptr: *mut u8, _npages: usize) {
        unsafe {
            VirtualFree(ptr as *mut _, 0, MEM_RELEASE);
        }
    }

    fn page_size() -> usize {
        PAGE_SIZE
    }
}
