use crate::xmem::PageAllocator;
use crate::{bus::bus::*, cpu::Exception};

use crate::*;

pub const RAM_BEGIN_ADDR: BusType = 0x80000000;

pub struct Ram {
    pub mem: *mut u8,
    len: usize,
}

impl Ram {
    pub fn new(mem_data: Vec<u8>) -> Self {
        let mem = PageAllocator::allocate_pages_at(
            RAM_BEGIN_ADDR as usize,
            mem_data.len() / PageAllocator::get_page_size(),
        )
        .unwrap();

        unsafe {
            std::ptr::copy_nonoverlapping(
                mem_data.as_ptr(),
                mem as *mut u8,
                mem_data.len() * std::mem::size_of::<u8>(),
            );
        }

        Self {
            mem,
            len: mem_data.len(),
        }
    }
}

impl BusDevice for Ram {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        let data = ptr_direct_load!(addr as *mut u8, size);

        Ok(data)
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        ptr_direct_store!(addr as *mut u8, data, size);

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        return RAM_BEGIN_ADDR;
    }

    fn get_end_addr(&self) -> BusType {
        return self.get_begin_addr() + self.len as BusType;
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, Exception> {
        Ok(addr as *mut u8)
    }

    fn tick_from_main_thread(&mut self) {}
}
