use crate::xmem::PageAllocator;
use crate::{bus::bus::*, cpu::Exception};

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
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        // let mut data: BusType = 0;

        // unsafe {
        //     std::ptr::copy_nonoverlapping(
        //         self.mem.as_ptr().add(adj_addr),
        //         &mut data as *mut BusType as *mut u8,
        //         size as usize / 8,
        //     );
        // }

        let data = unsafe {
            match size {
                8 => *(self.mem.add(adj_addr) as *const u8) as BusType,
                16 => *(self.mem.add(adj_addr) as *const u16) as BusType,
                32 => *(self.mem.add(adj_addr) as *const u32) as BusType,
                _ => panic!("Invalid size"),
            }
        };

        Ok(data)
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        // unsafe {
        //     std::ptr::copy_nonoverlapping(
        //         &data as *const BusType as *const u8,
        //         self.mem.as_mut_ptr().add(adj_addr),
        //         size as usize / 8,
        //     );
        // }

        unsafe {
            match size {
                8 => *(self.mem.add(adj_addr) as *mut u8) = data as u8,
                16 => *(self.mem.add(adj_addr) as *mut u16) = data as u16,
                32 => *(self.mem.add(adj_addr) as *mut u32) = data as u32,
                _ => panic!("Invalid size"),
            }
        };

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
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        unsafe { Ok(self.mem.add(adj_addr)) }
    }

    fn tick_from_main_thread(&mut self) {}
}
