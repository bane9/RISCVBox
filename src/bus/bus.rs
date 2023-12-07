use crate::bus::mmu::*;
use crate::cpu::*;

use super::ram::RAM_BEGIN_ADDR;
use super::ramfb::RAMFB_BEGIN_ADDR;

pub type BusType = u32;

#[macro_export]
#[cfg(feature = "no_unaligned_mem_access")]
macro_rules! ptr_direct_load {
    ($ptr:expr, $size:expr) => {{
        let ret: u32 = 0;

        unsafe {
            std::ptr::copy_nonoverlapping(
                $ptr,
                &ret as *const u32 as *mut u8,
                ($size as usize) / 8,
            );
        }

        ret
    }};
}

#[macro_export]
#[cfg(feature = "no_unaligned_mem_access")]
macro_rules! ptr_direct_store {
    ($ptr:expr, $data:expr, $size:expr) => {{
        unsafe {
            std::ptr::copy_nonoverlapping(
                &$data as *const u32 as *const u8,
                $ptr,
                ($size as usize) / 8,
            );
        }
    }};
}

#[macro_export]
#[cfg(not(feature = "no_unaligned_mem_access"))]
macro_rules! ptr_direct_load {
    ($ptr:expr, $size:expr) => {
        unsafe {
            match $size {
                8 => *(($ptr) as *const u8) as u32,
                16 => *(($ptr) as *const u16) as u32,
                32 => *(($ptr) as *const u32) as u32,
                _ => panic!("Invalid size"),
            }
        }
    };
}

#[macro_export]
#[cfg(not(feature = "no_unaligned_mem_access"))]
macro_rules! ptr_direct_store {
    ($ptr:expr, $data:expr, $size:expr) => {
        unsafe {
            match $size {
                8 => *(($ptr) as *mut u8) = $data as u8,
                16 => *(($ptr) as *mut u16) = $data as u16,
                32 => *(($ptr) as *mut u32) = $data as u32,
                _ => panic!("Invalid size"),
            }
        }
    };
}

pub trait BusDevice {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception>;
    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception>;
    fn get_begin_addr(&self) -> BusType;
    fn get_end_addr(&self) -> BusType;
    fn tick_core_local(&mut self);
    fn tick_from_main_thread(&mut self);
    fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, Exception>;
}

pub struct Bus {
    devices: Vec<Box<dyn BusDevice>>,
    ram_ptr: *mut u8,
    ram_end_addr: usize,

    fb_ptr: *mut u8,
    fb_end_addr: usize,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            devices: Vec::new(),
            ram_ptr: std::ptr::null_mut(),
            ram_end_addr: 0,

            fb_ptr: std::ptr::null_mut(),
            fb_end_addr: 0,
        }
    }

    pub fn add_device(&mut self, device: Box<dyn BusDevice>) {
        self.devices.push(device);
    }

    pub fn set_ram_ptr(&mut self, ptr: *mut u8, end_addr: usize) {
        self.ram_ptr = ptr;
        self.ram_end_addr = end_addr;
    }

    pub fn set_fb_ptr(&mut self, ptr: *mut u8, end_addr: usize) {
        self.fb_ptr = ptr;
        self.fb_end_addr = end_addr;
    }

    pub fn translate(
        &self,
        addr: BusType,
        mmu: &Sv32Mmu,
        access_type: AccessType,
    ) -> Result<BusType, Exception> {
        return mmu.translate(addr, access_type);
    }

    pub fn load(
        &mut self,
        addr: BusType,
        size: BusType,
        mmu: &Sv32Mmu,
    ) -> Result<BusType, Exception> {
        if !mmu.is_active() {
            return self.load_nommu(addr, size);
        }

        let phys_addr = self.translate(addr, mmu, AccessType::Load)?;

        let res = self.load_nommu(phys_addr, size);

        if res.is_err() && mmu.is_active() {
            return Err(Exception::LoadPageFault(addr));
        }

        res
    }

    pub fn fetch(
        &mut self,
        addr: BusType,
        size: BusType,
        mmu: &Sv32Mmu,
    ) -> Result<BusType, Exception> {
        if !mmu.is_active() {
            return self.fetch_nommu(addr, size);
        }

        let phys_addr = self.translate(addr, mmu, AccessType::Fetch)?;

        let res = self.load_nommu(phys_addr, size);

        if res.is_err() {
            if !mmu.is_active() {
                return Err(Exception::InstructionAccessFault(addr));
            } else {
                return Err(Exception::InstructionPageFault(addr));
            }
        }

        res
    }

    pub fn store(
        &mut self,
        addr: BusType,
        data: BusType,
        size: BusType,
        mmu: &Sv32Mmu,
    ) -> Result<(), Exception> {
        if !mmu.is_active() {
            return self.store_nommu(addr, data, size);
        }

        let phys_addr = self.translate(addr, mmu, AccessType::Store)?;

        let res = self.store_nommu(phys_addr, data, size);

        if res.is_err() && mmu.is_active() {
            return Err(Exception::StorePageFault(addr));
        }

        res
    }

    pub fn load_nommu(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        if addr >= RAM_BEGIN_ADDR && addr < self.ram_end_addr as u32 {
            let data = ptr_direct_load!(addr as *mut u8, size);

            return Ok(data);
        }

        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.load(addr, size);
            }
        }

        Err(Exception::LoadAccessFault(addr))
    }

    pub fn fetch_nommu(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        if addr >= RAM_BEGIN_ADDR && addr < self.ram_end_addr as u32 {
            let data = ptr_direct_load!(addr as *mut u8, size);

            return Ok(data);
        }

        let res = self.load_nommu(addr, size);

        if res.is_err() {
            return Err(Exception::InstructionAccessFault(addr));
        }

        res
    }

    pub fn store_nommu(
        &mut self,
        addr: BusType,
        data: BusType,
        size: BusType,
    ) -> Result<(), Exception> {
        if addr >= RAM_BEGIN_ADDR && addr < self.ram_end_addr as u32 {
            ptr_direct_store!(addr as *mut u8, data, size);

            return Ok(());
        } else if addr >= RAMFB_BEGIN_ADDR as u32 && addr < self.fb_end_addr as u32 {
            ptr_direct_store!(addr as *mut u8, data, size);

            return Ok(());
        }

        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.store(addr, data, size);
            }
        }

        Err(Exception::StoreAccessFault(addr))
    }

    pub fn tick_core_local(&mut self) {
        for device in &mut self.devices {
            device.tick_core_local();
        }
    }

    pub fn tick_from_main_thread(&mut self) {
        for device in &mut self.devices {
            device.tick_from_main_thread();
        }
    }

    pub fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, Exception> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.get_ptr(addr);
            }
        }

        Err(Exception::LoadAccessFault(addr))
    }
}

static mut BUS: Option<Bus> = None;

pub fn get_bus() -> &'static mut Bus {
    unsafe {
        if BUS.is_none() {
            BUS = Some(Bus::new());
        }
        BUS.as_mut().unwrap()
    }
}
