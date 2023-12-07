#[cfg(windows)]
pub mod win32_page_allocator {
    use crate::util;
    use crate::xmem::{AllocationError, PageState};
    use std::ptr;
    use winapi::um::memoryapi::{VirtualAlloc, VirtualFree, VirtualProtect};
    use winapi::um::winnt::{
        MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_EXECUTE_READ, PAGE_NOACCESS, PAGE_READWRITE,
    };

    const PAGE_SIZE: usize = 4096;

    pub fn allocate_pages(npages: usize) -> Result<*mut u8, AllocationError> {
        let size = npages * PAGE_SIZE;

        unsafe {
            let ptr = VirtualAlloc(ptr::null_mut(), size, MEM_COMMIT, PAGE_READWRITE) as *mut u8;

            if ptr == ptr::null_mut() {
                Err(AllocationError::UnknownError)
            } else {
                Ok(ptr)
            }
        }
    }

    pub fn allocate_pages_at(address: usize, npages: usize) -> Result<*mut u8, AllocationError> {
        let size = npages * PAGE_SIZE;

        unsafe {
            let ptr = VirtualAlloc(
                address as *mut _,
                size,
                MEM_COMMIT | MEM_RESERVE,
                PAGE_READWRITE,
            ) as *mut u8;

            if ptr != address as *mut u8 {
                panic!("VirtualAlloc returned a different address than requested (requested: {:p}, returned: {:p})", address as *mut u8, ptr);
            }

            if ptr == ptr::null_mut() {
                Err(AllocationError::UnknownError)
            } else {
                Ok(ptr)
            }
        }
    }

    pub fn free_pages(ptr: *mut u8, npages: usize) {
        let size = npages * PAGE_SIZE;

        unsafe {
            VirtualFree(ptr as *mut _, size, MEM_RELEASE);
        }
    }

    pub fn mark_page(
        ptr: *mut u8,
        npages: usize,
        pagestate: PageState,
    ) -> Result<(), AllocationError> {
        let size = npages * PAGE_SIZE;
        let mut old_protect = 0;
        let protect = match pagestate {
            PageState::ReadWrite => PAGE_READWRITE,
            PageState::ReadExecute => PAGE_EXECUTE_READ,
            PageState::Invalid => PAGE_NOACCESS,
        };

        let result = unsafe { VirtualProtect(ptr as *mut _, size, protect, &mut old_protect) };

        if result == 0 {
            Err(AllocationError::UnknownError)
        } else {
            Ok(())
        }
    }

    pub fn realloc_pages(
        ptr: *mut u8,
        npages: usize,
        new_size: usize,
    ) -> Result<*mut u8, AllocationError> {
        let new_npages = util::align_up(new_size, PAGE_SIZE) / PAGE_SIZE;

        if new_npages == npages {
            return Ok(ptr);
        }

        let new_ptr = allocate_pages(new_npages);

        if new_ptr.is_err() {
            return new_ptr;
        }

        let new_ptr = new_ptr.unwrap();

        unsafe {
            std::ptr::copy_nonoverlapping(
                ptr,
                new_ptr,
                std::cmp::min(npages, new_npages) * PAGE_SIZE,
            );
        }

        free_pages(ptr, npages);

        Ok(new_ptr)
    }

    pub fn get_page_size() -> usize {
        PAGE_SIZE
    }
}

#[cfg(posix)]
pub mod posix_page_allocator {
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

    pub fn allocate_pages(npages: usize) -> Result<*mut u8, AllocationError> {
        unsafe {
            let ptr = mmap(
                ptr::null_mut(),
                npages * PAGE_SIZE,
                PROT_READ | PROT_WRITE,
                MAP_PRIVATE | MAP_ANON,
                -1,
                0,
            );

            if ptr == ptr::null_mut() {
                Err(AllocationError::UnknownError)
            } else {
                Ok(ptr as *mut u8)
            }
        }
    }

    pub fn free_pages(ptr: *mut u8, npages: usize) {}

    pub fn mark_page(
        ptr: *mut u8,
        npages: usize,
        pagestate: PageState,
    ) -> Result<(), AllocationError> {
        let size = npages * PAGE_SIZE;
        let mut old_protect = 0;
        let protect = match pagestate {
            PageState::ReadWrite => PROT_READ | PROT_WRITE,
            PageState::ReadExecute => PROT_EXEC | PROT_READ,
            PageState::Invalid => PROT_NONE,
        };

        let result = unsafe { mprotect(ptr as *mut _, size, protect) };

        if result != 0 {
            Err(AllocationError::UnknownError)
        } else {
            Ok(())
        }
    }

    pub fn realloc_pages(
        ptr: *mut u8,
        npages: usize,
        new_size: usize,
    ) -> Result<*mut u8, AllocationError> {
        let new_npages = util::align_up(new_size, PAGE_SIZE) / PAGE_SIZE;

        if new_npages == npages {
            return Ok(ptr);
        }

        let new_ptr = allocate_pages(new_npages);

        if new_ptr.is_err() {
            return new_ptr;
        }

        let new_ptr = new_ptr.unwrap();

        unsafe {
            std::ptr::copy_nonoverlapping(
                ptr,
                new_ptr,
                std::cmp::min(npages, new_npages) * PAGE_SIZE,
            );
        }

        free_pages(ptr, npages);

        Ok(new_ptr)
    }

    pub fn get_page_size() -> usize {
        PAGE_SIZE
    }
}
