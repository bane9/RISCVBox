use crate::xmem::page_common::*;

extern crate winapi;

use crate::util;
use std::ptr;
use winapi::um::memoryapi::{VirtualAlloc, VirtualFree, VirtualProtect};
use winapi::um::winnt::{
    MEM_COMMIT, MEM_RELEASE, PAGE_EXECUTE_READ, PAGE_NOACCESS, PAGE_READWRITE,
};

const PAGE_SIZE: usize = 4096;

pub struct Win32Allocator {
    ptr: *mut u8,
    npages: usize,
    offset: usize,
    state: PageState,
}

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

impl CodePage for Win32Allocator {
    fn new() -> Self {
        let ptr = unsafe {
            VirtualAlloc(ptr::null_mut(), 32 * PAGE_SIZE, MEM_COMMIT, PAGE_READWRITE) as *mut u8
        };

        assert!(ptr != ptr::null_mut());

        Win32Allocator {
            ptr,
            npages: 32,
            offset: 0,
            state: PageState::ReadWrite,
        }
    }

    fn push(&mut self, data: &[u8]) -> Result<(), AllocationError> {
        if self.offset + data.len() > self.npages * PAGE_SIZE {
            unsafe {
                let npages = std::cmp::max(
                    util::align_up(self.offset + data.len(), PAGE_SIZE) / PAGE_SIZE,
                    self.npages * 2, // A logarithmic growth strategy may be better
                );

                let ptr = VirtualAlloc(
                    ptr::null_mut(),
                    npages * PAGE_SIZE,
                    MEM_COMMIT,
                    PAGE_READWRITE,
                ) as *mut u8;

                if self.ptr == ptr::null_mut() {
                    return Err(AllocationError::OutOfMemory);
                }

                ptr::copy_nonoverlapping(self.ptr, ptr, self.npages * PAGE_SIZE);

                VirtualFree(self.ptr as *mut _, 0, MEM_RELEASE);

                self.ptr = ptr;
                self.npages = npages;
            }
        }

        self.mark_rw()?;

        unsafe {
            ptr::copy_nonoverlapping(data.as_ptr(), self.ptr.add(self.offset), data.len());
        }

        self.offset += data.len();

        Ok(())
    }

    fn mark_rw(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::ReadWrite {
            return Ok(());
        }

        win32_mark_page(self.ptr, self.npages, PAGE_READWRITE)?;

        self.state = PageState::ReadWrite;

        Ok(())
    }

    fn mark_rx(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::ReadExecute {
            return Ok(());
        }

        win32_mark_page(self.ptr, self.npages, PAGE_EXECUTE_READ)?;

        self.state = PageState::ReadExecute;

        Ok(())
    }

    fn mark_invalid(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::Invalid {
            return Ok(());
        }

        win32_mark_page(self.ptr, self.npages, PAGE_NOACCESS)?;

        self.state = PageState::Invalid;

        Ok(())
    }

    fn dealloc(&mut self) {
        unsafe {
            VirtualFree(self.ptr as *mut _, 0, MEM_RELEASE);
        }
    }

    fn as_ptr(&self) -> *mut u8 {
        self.ptr
    }

    fn size(&self) -> usize {
        self.offset
    }

    fn npages(&self) -> usize {
        self.npages
    }

    fn state(&self) -> PageState {
        self.state
    }
}
