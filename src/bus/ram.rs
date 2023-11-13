use crate::{bus::bus::*, cpu::Exception};

pub struct Ram {
    pub mem: Vec<u8>,
}

impl Ram {
    pub fn new(mem: Vec<u8>) -> Self {
        Self { mem }
    }
}

impl BusDevice for Ram {
    fn read(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        match size {
            8 => Ok(self.mem[adj_addr] as BusType),
            16 => Ok(u16::from_le_bytes([self.mem[adj_addr], self.mem[adj_addr + 1]]) as BusType),
            32 => Ok(u32::from_le_bytes([
                self.mem[adj_addr],
                self.mem[adj_addr + 1],
                self.mem[adj_addr + 2],
                self.mem[adj_addr + 3],
            ]) as BusType),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
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
