use crate::bus::mmu::*;
use crate::cpu::*;

use super::plic::{Plic, PLIC_BASE};
use super::ram::RAM_BEGIN_ADDR;
use super::tlb::{tlb_fetch_instr, tlb_fetch_load, tlb_fetch_store};

pub type BusType = u32;

use csr::CsrType;
use vm_fdt::FdtWriter;

#[macro_export]
macro_rules! ptr_direct_load {
    ($ptr:expr, $size:expr) => {
        unsafe {
            match $size {
                8 => *(($ptr) as *const u8) as u32,
                16 => *(($ptr) as *const u16) as u32,
                32 => *(($ptr) as *const u32) as u32,
                _ => {
                    println!("Invalid size: {}", $size);
                    std::process::exit(1);
                }
            }
        }
    };
}

#[macro_export]
macro_rules! ptr_direct_store {
    ($ptr:expr, $data:expr, $size:expr) => {
        unsafe {
            match $size {
                8 => *(($ptr) as *mut u8) = $data as u8,
                16 => *(($ptr) as *mut u16) = $data as u16,
                32 => *(($ptr) as *mut u32) = $data as u32,
                _ => {
                    println!("Invalid size: {}", $size);
                    std::process::exit(1);
                }
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
    fn tick_async(&mut self, cpu: &mut Cpu) -> Option<u32>;
    fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, Exception>;
    fn describe_fdt(&self, fdt: &mut FdtWriter);
}

pub struct Bus {
    devices: Vec<Box<dyn BusDevice>>,
    ram_ptr: *mut u8,
    ram_end_addr: usize,

    fb_ptr: *mut u8,
    fb_end_addr: usize,

    plic_ptr: *mut Plic,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            devices: Vec::new(),
            ram_ptr: std::ptr::null_mut(),
            ram_end_addr: 0,

            fb_ptr: std::ptr::null_mut(),
            fb_end_addr: 0,

            plic_ptr: std::ptr::null_mut(),
        }
    }

    pub fn add_device(&mut self, device: Box<dyn BusDevice>) {
        if device.get_begin_addr() == PLIC_BASE {
            self.plic_ptr = device.as_ref() as *const dyn BusDevice as *mut Plic;
        }

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
        &mut self,
        addr: BusType,
        mmu: &mut Sv32Mmu,
        access_type: AccessType,
    ) -> Result<BusType, Exception> {
        return mmu.translate(addr, access_type);
    }

    pub fn load(
        &mut self,
        addr: BusType,
        size: BusType,
        mmu: &mut Sv32Mmu,
    ) -> Result<BusType, Exception> {
        if !mmu.is_active() {
            return self.load_nommu(addr, size);
        }

        let phys_addr = if let Some(phys_addr) = tlb_fetch_load(addr) {
            phys_addr
        } else {
            self.translate(addr, mmu, AccessType::Load)?
        };

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
        mmu: &mut Sv32Mmu,
    ) -> Result<BusType, Exception> {
        if !mmu.is_active() {
            return self.fetch_nommu(addr, size);
        }

        let phys_addr = if let Some(phys_addr) = tlb_fetch_instr(addr) {
            phys_addr
        } else {
            self.translate(addr, mmu, AccessType::Fetch)?
        };

        let res = self.load_nommu(phys_addr, size);

        if res.is_err() {
            if !mmu.is_active() {
                return Err(Exception::InstructionAccessFault(addr));
            }

            return Err(Exception::InstructionPageFault(addr));
        }

        res
    }

    pub fn store(
        &mut self,
        addr: BusType,
        data: BusType,
        size: BusType,
        mmu: &mut Sv32Mmu,
    ) -> Result<(), Exception> {
        if !mmu.is_active() {
            return self.store_nommu(addr, data, size);
        }

        let phys_addr = if let Some(phys_addr) = tlb_fetch_store(addr) {
            phys_addr
        } else {
            self.translate(addr, mmu, AccessType::Store)?
        };

        let res = self.store_nommu(phys_addr, data, size);

        if res.is_err() && mmu.is_active() {
            return Err(Exception::StorePageFault(addr));
        }

        res
    }

    pub fn load_nommu(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        if self.is_dram_addr(addr) {
            let offset = addr - RAM_BEGIN_ADDR;
            let ptr = unsafe { self.ram_ptr.add(offset as usize) };

            return Ok(ptr_direct_load!(ptr, size));
        }

        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.load(addr, size);
            }
        }

        Err(Exception::LoadAccessFault(addr))
    }

    pub fn fetch_nommu(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
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
        if self.is_dram_addr(addr) {
            let offset = addr - RAM_BEGIN_ADDR;
            let ptr = unsafe { self.ram_ptr.add(offset as usize) };

            ptr_direct_store!(ptr, data, size);

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

    pub fn tick_async(&mut self, cpu: &mut cpu::Cpu) {
        let mut new_mip: CsrType = 0;

        for device in &mut self.devices {
            if let Some(irq) = device.tick_async(cpu) {
                new_mip |= 1 << irq;
                break;
            }
        }

        if new_mip != 0 {
            cpu.csr.or_mip_atomic(new_mip);
            cpu.has_pending_interrupt
                .store(1, std::sync::atomic::Ordering::Release);
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

    pub fn get_plic(&mut self) -> &'static mut Plic {
        return unsafe { &mut *self.plic_ptr };
    }

    pub fn get_ram_end_addr(&self) -> usize {
        self.ram_end_addr
    }

    pub fn is_dram_addr(&self, addr: BusType) -> bool {
        addr >= RAM_BEGIN_ADDR && addr < self.ram_end_addr as u32
    }

    pub fn describe_fdts(&self, fdt: &mut FdtWriter) {
        for device in &self.devices {
            device.describe_fdt(fdt);
        }
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

pub fn cleanup() {
    unsafe {
        BUS = None;
    }
}
