use crate::cpu::Exception;

pub type BusType = u32;

pub trait BusDevice {
    fn read(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception>;
    fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception>;
    fn get_begin_addr(&self) -> BusType;
    fn get_end_addr(&self) -> BusType;
    fn tick(&mut self);
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

    pub fn translate(&self, addr: BusType) -> Result<BusType, Exception> {
        return Ok(addr);
    }

    pub fn read(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.read(addr, size);
            }
        }

        Err(Exception::LoadAccessFault(addr))
    }

    pub fn fetch(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        self.read(addr, size)
    }

    pub fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.write(addr, data, size);
            }
        }

        Err(Exception::StoreAccessFault(addr))
    }

    pub fn tick(&mut self) {
        for device in &mut self.devices {
            device.tick();
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
