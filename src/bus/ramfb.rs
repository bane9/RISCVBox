use crate::{bus::bus::*, cpu::Exception};

pub const RAMFB_BEGIN_ADDR: BusType = 0x1d385000;

pub struct RamFB {
    pub mem: Vec<u8>,

    pub width: usize,
    pub height: usize,
    pub bpp: usize,
}

impl RamFB {
    pub fn new(width: usize, height: usize, bpp: usize) -> Self {
        Self {
            mem: vec![0; width * height * (bpp / 8)],
            width,
            height,
            bpp,
        }
    }

    pub fn get_fb_ptr(&mut self) -> *mut u8 {
        self.mem.as_mut_ptr()
    }
}

impl BusDevice for RamFB {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        let mut data: BusType = 0;

        unsafe {
            std::ptr::copy_nonoverlapping(
                self.mem.as_ptr().add(adj_addr),
                &mut data as *mut BusType as *mut u8,
                size as usize / 8,
            );
        }

        Ok(data)
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        unsafe {
            std::ptr::copy_nonoverlapping(
                &data as *const BusType as *const u8,
                self.mem.as_mut_ptr().add(adj_addr),
                size as usize / 8,
            );
        }

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        return RAMFB_BEGIN_ADDR;
    }

    fn get_end_addr(&self) -> BusType {
        return self.get_begin_addr() + self.mem.len() as BusType;
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        unsafe { Ok(self.mem.as_mut_ptr().add(adj_addr)) }
    }

    fn tick_from_main_thread(&mut self) {}
}
