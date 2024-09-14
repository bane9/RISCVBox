use xmem::PageState;

use crate::bus::bus::*;
use crate::xmem::PageAllocator;

use crate::*;

pub const RAM_BEGIN_ADDR: BusType = 0x80000000;

pub struct Ram {
    len: usize,
}

static mut RAM: *mut u8 = std::ptr::null_mut();

fn init_ram_once(mem_data: Vec<u8>) {
    unsafe {
        if !RAM.is_null() {
            PageAllocator::mark_page(
                RAM,
                mem_data.len() / PageAllocator::get_page_size(),
                PageState::ReadWrite,
            )
            .unwrap();

            std::ptr::copy_nonoverlapping(mem_data.as_ptr(), RAM, mem_data.len());

            return;
        }

        RAM = PageAllocator::allocate_pages_at(
            RAM_BEGIN_ADDR as usize,
            mem_data.len() / PageAllocator::get_page_size(),
        )
        .unwrap();

        std::ptr::copy_nonoverlapping(mem_data.as_ptr(), RAM, mem_data.len());
    }
}

impl Ram {
    pub fn new(mem_data: Vec<u8>) -> Self {
        let len = mem_data.len();

        init_ram_once(mem_data);

        Self { len }
    }
}

impl BusDevice for Ram {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, cpu::Exception> {
        let data = ptr_direct_load!(addr as *mut u8, size);

        Ok(data)
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), cpu::Exception> {
        ptr_direct_store!(addr as *mut u8, data, size);

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        RAM_BEGIN_ADDR
    }

    fn get_end_addr(&self) -> BusType {
        self.get_begin_addr() + self.len as BusType
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, cpu::Exception> {
        Ok(addr as *mut u8)
    }

    fn tick_from_main_thread(&mut self) {}

    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        None
    }

    fn describe_fdt(&self, _fdt: &mut vm_fdt::FdtWriter) {}
}
