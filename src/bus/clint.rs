use crate::bus::*;
use crate::cpu::*;
use crate::frontend::exec_core::INSN_SIZE;
use crate::util;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

pub const CLINT_ADDR: BusType = 0x2000000;
const CLINT_END: BusType = CLINT_ADDR + 0x10000;

const MSIP: BusType = CLINT_ADDR;
const MSIP_END: BusType = MSIP + INSN_SIZE as BusType;

const MTIMECMP: BusType = CLINT_ADDR + 0x4000;
const MTIMECMP_END: BusType = MTIMECMP + INSN_SIZE as BusType;

const MTIME: BusType = CLINT_ADDR + 0xbff8;
const MTIME_END: BusType = MTIME + INSN_SIZE as BusType;

const CLINT_IRQN: usize = 0;

pub struct ClintData {
    pub msip: BusType,
    pub mtimecmp: BusType,
}

impl ClintData {
    pub fn new() -> ClintData {
        ClintData {
            msip: 0,
            mtimecmp: 0,
        }
    }
}

lazy_static! {
    static ref CLINTS: Mutex<HashMap<usize, ClintData>> = Mutex::new(HashMap::new());
}

fn get_clint(thread_id: usize) -> &'static mut ClintData {
    let mut map = CLINTS.lock().unwrap();

    if !map.contains_key(&thread_id) {
        let clint = ClintData::new();
        map.insert(thread_id, clint);
    }

    unsafe {
        let clint = map.get_mut(&thread_id).unwrap();
        let clint = clint as *mut ClintData;

        &mut *clint
    }
}
static mut ATOMIC_CNT: AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
pub struct Clint;

impl Clint {
    pub fn new() -> Clint {
        Clint {}
    }

    pub fn get_remaining_time_ms() -> u64 {
        let clint = get_clint(cpu::get_cpu().core_id as usize);

        let mtime = util::ms_since_program_start() as BusType;

        let diff = clint.mtimecmp as i64 - mtime as i64;

        if diff < 0 {
            0
        } else {
            diff as u64
        }
    }

    pub fn tick(clint_data: &mut ClintData, cpu: &mut cpu::Cpu) -> bool {
        let mtime = util::ms_since_program_start() as BusType;

        if (clint_data.msip & 1) != 0 {
            cpu.csr
                .write_bit(csr::register::MIP, csr::bits::MSIP_BIT, true);

            return true;
        }

        if mtime >= clint_data.mtimecmp {
            cpu.csr
                .write_bit(csr::register::MIP, csr::bits::MTIP_BIT, true);

            cpu.pending_interrupt_number = CLINT_IRQN as CpuReg;
            return true;
        }

        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::MTIP_BIT, false);

        false
    }
}

impl BusDevice for Clint {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        let clint = get_clint(cpu::get_cpu().core_id as usize);

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
                let mtime = util::ms_since_program_start() as BusType;
                let src = &mtime as *const BusType as *const u8;
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
        let clint = get_clint(cpu::get_cpu().core_id as usize);

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
            MTIME..=MTIME_END => {}
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
        let clint = get_clint(cpu::get_cpu().core_id as usize);

        Self::tick(clint, cpu::get_cpu());
    }

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}

    fn tick_async(&mut self, cpu: &mut cpu::Cpu) -> Option<u32> {
        // This mandates syncrhonization/atomics but I'll hope it'll be fine for now
        let clint = get_clint(cpu.core_id as usize);

        if Self::tick(clint, cpu) {
            Some(CLINT_IRQN as u32)
        } else {
            None
        }
    }
}
