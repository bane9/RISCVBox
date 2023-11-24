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

impl Clint {
    pub fn new() -> Clint {
        Clint {}
    }
}

impl BusDevice for Clint {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        let clint = get_clint();

        let mut out = 0 as BusType;

        match addr {
            MSIP..=MSIP_END => unsafe {
                let src = &clint.msip as *const BusType as *const u8;
                std::ptr::copy_nonoverlapping(
                    src,
                    &mut out as *mut u32 as *mut u8,
                    size as usize / 8,
                );
            },
            MTIMECMP..=MTIMECMP_END => unsafe {
                let src = &clint.mtimecmp as *const BusType as *const u8;
                std::ptr::copy_nonoverlapping(
                    src,
                    &mut out as *mut u32 as *mut u8,
                    size as usize / 8,
                );
            },
            MTIME..=MTIME_END => unsafe {
                let src = &clint.mtime as *const BusType as *const u8;
                std::ptr::copy_nonoverlapping(
                    src,
                    &mut out as *mut u32 as *mut u8,
                    size as usize / 8,
                );
            },
            _ => return Err(Exception::LoadAccessFault(addr)),
        };

        Ok(out)
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        let clint = get_clint();

        match addr {
            MSIP..=MSIP_END => unsafe {
                clint.msip = 0;
                let dst = &mut clint.msip as *mut BusType as *mut u8;
                std::ptr::copy_nonoverlapping(
                    &data as *const u32 as *const u8,
                    dst,
                    size as usize / 8,
                );
            },
            MTIMECMP..=MTIMECMP_END => unsafe {
                clint.mtimecmp = 0;
                let dst = &mut clint.mtimecmp as *mut BusType as *mut u8;
                std::ptr::copy_nonoverlapping(
                    &data as *const u32 as *const u8,
                    dst,
                    size as usize / 8,
                );
            },
            MTIME..=MTIME_END => unsafe {
                clint.mtime = 0;
                let dst = &mut clint.mtime as *mut BusType as *mut u8;
                std::ptr::copy_nonoverlapping(
                    &data as *const u32 as *const u8,
                    dst,
                    size as usize / 8,
                );
            },
            _ => return Err(Exception::StoreAccessFault(addr)),
        };

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
