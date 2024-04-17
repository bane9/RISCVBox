use crate::*;
use crate::{bus::bus::*, xmem::PageAllocator};

pub const RAMFB_BEGIN_ADDR: BusType = 0x1d380000;

pub struct RamFB {
    pub mem: *mut u8,
    len: usize,
}

impl RamFB {
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        let fb_len = width * height * (bpp / 8);
        let fb_len = util::align_up(fb_len, PageAllocator::get_page_size());

        let mem = PageAllocator::allocate_pages_at(
            RAMFB_BEGIN_ADDR as usize,
            fb_len / PageAllocator::get_page_size(),
        )
        .unwrap();

        Self { mem, len: fb_len }
    }

    pub fn get_fb_ptr(&mut self) -> *mut u8 {
        self.mem
    }
}

impl BusDevice for RamFB {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, cpu::Exception> {
        let data = ptr_direct_load!(addr as *mut u8, size);

        Ok(data)
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), cpu::Exception> {
        ptr_direct_store!(addr as *mut u8, data, size);

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        RAMFB_BEGIN_ADDR
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
}
