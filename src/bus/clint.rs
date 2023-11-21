use crate::bus::*;
use crate::cpu::*;
use crate::frontend::exec_core::INSN_SIZE;
use chrono::Utc;
use std::cell::RefCell;

pub const CLINT_ADDR: BusType = 0x2000000;
const CLINT_END: BusType = CLINT_ADDR + 0x10000;

const MSIP: BusType = CLINT_ADDR;
const MSIP_END: BusType = MSIP + INSN_SIZE as BusType;

const MTIMECMP: BusType = CLINT_ADDR + 0x4000;
const MTIMECMP_END: BusType = MTIMECMP + INSN_SIZE as BusType;

const MTIME: BusType = CLINT_ADDR + 0xbff8;
const MTIME_END: BusType = MTIME + INSN_SIZE as BusType;

pub struct ClintData {
    pub msip: BusType,
    pub mtimecmp: BusType,
    pub mtime: BusType,
    pub start_time: i64,
}

impl ClintData {
    pub fn new() -> ClintData {
        ClintData {
            msip: 0,
            mtimecmp: 0,
            mtime: 0,
            start_time: Utc::now().timestamp_millis(),
        }
    }
}

thread_local! {
    static CLINT: RefCell<ClintData> = RefCell::new(ClintData::new());
}

fn get_clint() -> &'static mut ClintData {
    CLINT.with(|clint| unsafe { &mut *clint.as_ptr() })
}

pub struct Clint;

impl BusDevice for Clint {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        let clint = get_clint();

        let (reg, offset) = match addr {
            MSIP..=MSIP_END => (clint.msip, addr - MSIP),
            MTIMECMP..=MTIMECMP_END => (clint.mtimecmp, addr - MTIMECMP),
            MTIME..=MTIME_END => (clint.mtime, addr - MTIME),
            _ => return Err(Exception::LoadAccessFault(addr)),
        };

        match size {
            8 => Ok((reg >> (offset * 8)) & 0xff),
            16 => Ok((reg >> (offset * 8)) & 0xffff),
            32 => Ok((reg >> (offset * 8)) & 0xffffffff),
            _ => return Err(Exception::LoadAccessFault(addr)),
        }
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        let clint = get_clint();

        let (mut reg, offset) = match addr {
            MSIP..=MSIP_END => (clint.msip, addr - MSIP),
            MTIMECMP..=MTIMECMP_END => (clint.mtimecmp, addr - MTIMECMP),
            MTIME..=MTIME_END => (clint.mtime, addr - MTIME),
            _ => return Err(Exception::StoreAccessFault(addr)),
        };

        match size {
            8 => {
                reg = reg & (!(0xff << (offset * 8)));
                reg = reg | ((data & 0xff) << (offset * 8));
            }
            16 => {
                reg = reg & (!(0xffff << (offset * 8)));
                reg = reg | ((data & 0xffff) << (offset * 8));
            }
            32 => {
                reg = reg & (!(0xffffffff << (offset * 8)));
                reg = reg | ((data & 0xffffffff) << (offset * 8));
            }
            _ => return Err(Exception::StoreAccessFault(addr)),
        }

        match addr {
            MSIP..=MSIP_END => clint.msip = reg,
            MTIMECMP..=MTIMECMP_END => clint.mtimecmp = reg,
            MTIME..=MTIME_END => clint.mtime = reg,
            _ => return Err(Exception::StoreAccessFault(addr)),
        }

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        CLINT_ADDR as BusType
    }

    fn get_end_addr(&self) -> BusType {
        CLINT_END as BusType
    }

    fn tick_core_local(&mut self) {
        let cpu = get_cpu();
        let clint = get_clint();

        // TODO: Make it make sense
        clint.mtime = (Utc::now().timestamp_millis() - clint.start_time) as BusType;

        if (clint.msip & 1) != 0 {
            cpu.csr
                .write_bit(csr::register::MIP, csr::bits::MSIP_BIT, true);
        }

        if clint.mtime >= clint.mtimecmp {
            cpu.csr
                .write_bit(csr::register::MIP, csr::bits::MTIP_BIT, true);
        } else {
            cpu.csr
                .write_bit(csr::register::MIP, csr::bits::MTIP_BIT, false);
        }
    }

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}
}
