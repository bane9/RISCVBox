use crate::{
    bus::bus::*,
    cpu::{self, Exception},
    util,
};

pub const DTB_SIZE: usize = util::size_mib(2);
pub const DTB_BEGIN_ADDR: BusType = 0x1000;
pub const DTB_END: BusType = DTB_BEGIN_ADDR + DTB_SIZE as BusType;

pub struct Dtb {
    pub mem: Vec<u8>,
}

impl Dtb {
    pub fn new(mem: &[u8]) -> Self {
        let mut this = Self {
            mem: vec![0; DTB_SIZE],
        };

        this.mem[..mem.len()].copy_from_slice(mem);

        this
    }
}

impl BusDevice for Dtb {
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
        DTB_BEGIN_ADDR
    }

    fn get_end_addr(&self) -> BusType {
        DTB_END
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}

    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        None
    }
}
