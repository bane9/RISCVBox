use crate::cpu;

pub enum BusError {
    InvalidAddress,
    ReadFault,
    WriteFault,
    PageFault,
}

pub type BusType = u32;

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

    pub fn read(&mut self, addr: BusType, size: BusType) -> Result<BusType, BusError> {
        for device in &mut self.devices {
            if addr >= device.get_begin_addr() && addr < device.get_end_addr() {
                return device.read(addr, size);
            }
        }

        Err(BusError::InvalidAddress)
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

pub extern "C" fn c_bus_access(
    addr_reg: usize,
    data_reg: usize,
    imm_write_size_signed: usize,
    guest_pc: usize,
) -> usize {
    let bus = get_bus();
    let sign = imm_write_size_signed & 0x1;
    let write = (imm_write_size_signed >> 1) & 0x1;
    let size = ((imm_write_size_signed >> 2) & 0x8) * 8;
    let imm = (imm_write_size_signed >> 8) & 0xffff;
    let cpu = cpu::get_cpu();
    let addr = cpu.regs[addr_reg] + imm as BusType; // TODO: signextend
    let out: usize;

    if write == 0 {
        match bus.read(addr as BusType, size as BusType) {
            Ok(data) => out = data as usize,
            Err(_) => out = 0,
        }
    } else {
        match bus.write(addr as BusType, cpu.regs[data_reg], size as BusType) {
            Ok(_) => out = 1,
            Err(_) => out = 0,
        }
    }

    if out == 0 {
        return 0; // TODO: mark exception
    }

    cpu.regs[data_reg] = out as BusType;

    1
}
