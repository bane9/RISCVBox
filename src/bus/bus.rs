use crate::bus::mmu::*;
use crate::cpu::Exception;

pub type BusType = u32;

pub trait BusDevice {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception>;
    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception>;
    fn get_begin_addr(&self) -> BusType;
    fn get_end_addr(&self) -> BusType;
    fn tick(&mut self);
    fn get_ptr(&mut self, addr: BusType) -> Result<*mut u8, Exception>;
}

pub struct Bus {
    devices: Vec<Box<dyn BusDevice>>,
}

impl Bus {
    pub fn new() -> Bus {
        Bus {
            devices: Vec::new(),
        }
    }

    pub fn add_device(&mut self, device: Box<dyn BusDevice>) {
        self.devices.push(device);
    }

    pub fn translate(
        &self,
        addr: BusType,
        mmu: &Sv39Mmu,
        access_type: AccessType,
    ) -> Result<BusType, Exception> {
        return mmu.translate(addr, access_type);
    }

    pub fn load(
        &mut self,
        addr: BusType,
        size: BusType,
        mmu: &Sv39Mmu,
    ) -> Result<BusType, Exception> {
        let phys_addr = self.translate(addr, mmu, AccessType::Load)?;

        return self.load_nommu(phys_addr, size);
    }

    pub fn fetch(
        &mut self,
        addr: BusType,
        size: BusType,
        mmu: &Sv39Mmu,
    ) -> Result<BusType, Exception> {
        let phys_addr = self.translate(addr, mmu, AccessType::Fetch)?;

        let res = self.load_nommu(phys_addr, size);

        if res.is_err() {
            return Err(Exception::InstructionPageFault(addr));
        }

        res
    }

    pub fn store(
        &mut self,
        addr: BusType,
        data: BusType,
        size: BusType,
        mmu: &Sv39Mmu,
    ) -> Result<(), Exception> {
        let phys_addr = self.translate(addr, mmu, AccessType::Store)?;

        return self.store_nommu(phys_addr, data, size);
    }

    pub fn load_nommu(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.load(addr, size);
            }
        }

        Err(Exception::LoadAccessFault(addr))
    }

    pub fn fetch_nommu(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        self.load_nommu(addr, size)
    }

    pub fn store_nommu(
        &mut self,
        addr: BusType,
        data: BusType,
        size: BusType,
    ) -> Result<(), Exception> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.store(addr, data, size);
            }
        }

        Err(Exception::StoreAccessFault(addr))
    }

    pub fn tick(&mut self) {
        for device in &mut self.devices {
            device.tick();
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
