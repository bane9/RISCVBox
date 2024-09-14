use crate::{
    backend::{ReturnableHandler, ReturnableImpl},
    bus::bus::*,
    cpu,
};

pub const SYSCON_ADDR: BusType = 0x11100000;
pub const SYSCON_SIZE: BusType = 0x1000;

pub const SYSCON_POWEROFF: BusType = 0x5555;
pub const SYSCON_REBOOT: BusType = 0x7777;

pub struct Syscon;

impl Syscon {
    pub fn new() -> Self {
        Self {}
    }
}

static mut SHOULD_REBOOT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

fn set_should_reboot() {
    unsafe {
        SHOULD_REBOOT.store(true, std::sync::atomic::Ordering::Release);
    }
}

pub fn clear_should_reboot() {
    unsafe {
        SHOULD_REBOOT.store(false, std::sync::atomic::Ordering::Release);
    }
}

pub fn should_reboot() -> bool {
    unsafe { SHOULD_REBOOT.load(std::sync::atomic::Ordering::Acquire) }
}

impl BusDevice for Syscon {
    fn load(&mut self, _addr: BusType, _size: BusType) -> Result<BusType, cpu::Exception> {
        Ok(0)
    }

    fn store(
        &mut self,
        _addr: BusType,
        data: BusType,
        _size: BusType,
    ) -> Result<(), cpu::Exception> {
        if data == SYSCON_POWEROFF {
            std::process::exit(0);
        } else if data == SYSCON_REBOOT {
            set_should_reboot();
            cpu::get_cpu().exception = cpu::Exception::Reboot;
            ReturnableImpl::throw();
        }

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        SYSCON_ADDR
    }

    fn get_end_addr(&self) -> BusType {
        SYSCON_ADDR + SYSCON_SIZE
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, cpu::Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}

    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        None
    }

    fn describe_fdt(&self, _fdt: &mut vm_fdt::FdtWriter) {}
}
