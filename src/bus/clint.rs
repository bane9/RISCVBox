use crate::bus::*;
use crate::cpu::*;
use crate::frontend::exec_core::INSN_SIZE;
use crate::util;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::atomic::AtomicUsize;
use std::sync::Mutex;

use self::plic::PLIC_PHANDLE;

pub const CLINT_ADDR: BusType = 0x2000000;
const CLINT_SIZE: BusType = 0x10000;
const CLINT_END: BusType = CLINT_ADDR + CLINT_SIZE;

const MSIP: BusType = CLINT_ADDR;
const MSIP_END: BusType = MSIP + INSN_SIZE as BusType;

const MTIMECMP: BusType = CLINT_ADDR + 0x4000;
const MTIMECMP_END: BusType = MTIMECMP + INSN_SIZE as BusType;

const MTIME: BusType = CLINT_ADDR + 0xbff8;
const MTIME_END: BusType = MTIME + INSN_SIZE as BusType;

pub const CLINT_IRQN: u32 = 0;

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

fn get_time() -> BusType {
    let mtime = util::timebase_since_program_start();

    mtime as BusType
}
pub struct Clint;

impl Clint {
    pub fn new() -> Clint {
        Clint {}
    }

    pub fn tick(clint_data: &mut ClintData, cpu: &mut cpu::Cpu) -> Option<u32> {
        let mtime = get_time();

        if (clint_data.msip & 1) != 0 {
            cpu.pending_interrupt_number = CLINT_IRQN as CpuReg;

            return Some(csr::bits::MSIP_BIT as u32);
        }

        if mtime >= clint_data.mtimecmp {
            cpu.pending_interrupt_number = CLINT_IRQN as CpuReg;

            return Some(csr::bits::MTIP_BIT as u32);
        } else {
            cpu.csr.clear_bit_mip_atomic(csr::bits::MTIP_BIT);
        }

        None
    }
}

impl BusDevice for Clint {
    fn load(&mut self, addr: BusType, _size: BusType) -> Result<BusType, Exception> {
        let clint = get_clint(cpu::get_cpu().core_id as usize);

        match addr {
            MSIP..=MSIP_END => {
                return Ok(clint.msip);
            }
            MTIMECMP..=MTIMECMP_END => {
                return Ok(clint.mtimecmp);
            }
            MTIME..=MTIME_END => {
                return Ok(get_time());
            }
            _ => return Err(Exception::LoadAccessFault(addr)),
        };
    }

    fn store(&mut self, addr: BusType, data: BusType, _size: BusType) -> Result<(), Exception> {
        let clint = get_clint(cpu::get_cpu().core_id as usize);

        match addr {
            MSIP..=MSIP_END => {
                clint.msip = data;
            }
            MTIMECMP..=MTIMECMP_END => {
                clint.mtimecmp = data;
            }
            MTIME..=MTIME_END => {
                println!("mtime store\n\n\n")
            }
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
        let clint = get_clint(cpu.core_id as usize);

        Self::tick(clint, cpu)
    }

    fn describe_fdt(&self, fdt: &mut vm_fdt::FdtWriter) {
        let clint_node = fdt
            .begin_node(&util::fdt_node_addr_helper("clint", CLINT_ADDR))
            .unwrap();
        fdt.property_array_u32(
            "interrupts-extended",
            &[
                CPU_INTC_PHANDLE,
                PLIC_PHANDLE,
                CPU_INTC_PHANDLE,
                0x7 as BusType,
            ],
        )
        .unwrap();
        fdt.property_array_u32("reg", &[0x00, CLINT_ADDR, 0x00, CLINT_SIZE])
            .unwrap();
        fdt.property_string_list(
            "compatible",
            vec!["sifive,clint0".into(), "riscv,clint0".into()],
        )
        .unwrap();
        fdt.end_node(clint_node).unwrap();
    }
}
