pub type BusType = u32;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum BusError {
    InvalidAddress,
    ReadFault,
    WriteFault,
    PageFault,

    ForwardJumpFault(BusType),

    InvalidSize,
    None,
}

pub trait BusDevice {
    fn read(&mut self, addr: BusType, size: BusType) -> Result<BusType, BusError>;
    fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), BusError>;
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

    pub fn translate(&self, addr: BusType) -> Result<BusType, BusError> {
        return Ok(addr);
    }

    pub fn read(&mut self, addr: BusType, size: BusType) -> Result<BusType, BusError> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.read(addr, size);
            }
        }

        Err(BusError::InvalidAddress)
    }

    pub fn fetch(&mut self, addr: BusType, size: BusType) -> Result<BusType, BusError> {
        self.read(addr, size)
    }

    pub fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), BusError> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.write(addr, data, size);
            }
        }

        Err(BusError::InvalidAddress)
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
