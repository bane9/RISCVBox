use crate::{
    util,
    xmem::page_common::{AllocationError, CodePage},
};
use libc::{
    mmap, mprotect, munmap, MAP_ANON, MAP_PRIVATE, PROT_EXEC, PROT_NONE, PROT_READ, PROT_WRITE,
};
use std::ptr;

use super::PageState;

const PAGE_SIZE: usize = 4096;

pub struct PosixAllocator {
    ptr: *mut u8,
    npages: usize,
    offset: usize,
    state: PageState,
}

fn posix_mark_page(ptr: *mut u8, npages: usize, protect: i32) -> Result<(), AllocationError> {
    let size = npages * PAGE_SIZE;
    let result = unsafe { mprotect(ptr as *mut _, size, protect) };

    if result != 0 {
        Err(AllocationError::UnknownError)
    } else {
        Ok(())
    }
}

impl CodePage for PosixAllocator {
    fn new() -> Self {
        let ptr = unsafe {
            mmap(
                ptr::null_mut(),
                32 * PAGE_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANON,
                -1,
                0,
            )
        };

        assert!(ptr != ptr::null_mut());

        PosixAllocator {
            ptr: ptr as *mut u8,
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

                let ptr = mmap(
                    ptr::null_mut(),
                    npages * PAGE_SIZE,
                    PROT_READ | PROT_WRITE,
                    MAP_PRIVATE | MAP_ANON,
                    -1,
                    0,
                );

                assert!(ptr != ptr::null_mut());

                ptr::copy_nonoverlapping(self.ptr, ptr as *mut u8, self.npages * PAGE_SIZE);
                munmap(self.ptr as *mut _, self.npages * PAGE_SIZE);

                self.ptr = ptr as *mut u8;
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

        posix_mark_page(self.ptr, self.npages, PROT_READ | PROT_WRITE)?;

        self.state = PageState::ReadWrite;

        Ok(())
    }

    fn mark_rx(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::ReadExecute {
            return Ok(());
        }

        posix_mark_page(self.ptr, self.npages, PROT_READ | PROT_EXEC)?;

        self.state = PageState::ReadExecute;

        Ok(())
    }

    fn mark_invalid(&mut self) -> Result<(), AllocationError> {
        if self.state == PageState::Invalid {
            return Ok(());
        }

        posix_mark_page(self.ptr, self.npages, PROT_NONE)?;

        self.state = PageState::Invalid;

        Ok(())
    }

    fn dealloc(&mut self) {
        unsafe {
            munmap(self.ptr as *mut _, self.npages * PAGE_SIZE);
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
