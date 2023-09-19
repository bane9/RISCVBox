use crate::bus::bus::*;

pub struct Ram {
    pub mem: Vec<u8>,
}

impl Ram {
    pub fn new(mem: Vec<u8>) -> Self {
        Self { mem }
    }
}

impl BusDevice for Ram {
    fn read(&mut self, addr: BusType) -> Result<BusType, BusError> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        Ok(self.mem[adj_addr] as BusType)
    }

    fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), BusError> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        unsafe {
            std::ptr::copy_nonoverlapping(
                &data as *const BusType as *const u8,
                self.mem.as_mut_ptr().add(adj_addr),
                size as usize,
            );
        }

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        return 0;
    }

    fn get_end_addr(&self) -> BusType {
        return self.get_begin_addr() + self.mem.len() as BusType;
    }

    fn tick(&mut self) {}
}
