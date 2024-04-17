use crate::backend::target::core::BackendCoreImpl;
use crate::backend::{
    BackendCore, FastmemHandleType, ReturnStatus, ReturnableHandler, ReturnableImpl,
};
use crate::bus::clint::Clint;
use crate::bus::dtb::DTB_BEGIN_ADDR;
use crate::bus::mmu::{AccessType, Mmu};
use crate::bus::{self, BusType};
use crate::cpu::{self, csr, CpuReg};
use crate::cpu::{trap, RegName};
pub use crate::frontend::parse_core::*;
use crate::xmem::PageState;

use super::insn_lookup::InsnMappingData;

pub struct ExecCore {
    parse_core: ParseCore,
}

impl ExecCore {
    pub fn new() -> ExecCore {
        ExecCore {
            parse_core: ParseCore::new(),
        }
    }

    fn get_jit_ptr(&mut self) -> *mut u8 {
        let cpu = cpu::get_cpu();

        if cpu.exception != cpu::Exception::None {
            let int = if cpu.pending_interrupt.is_some() {
                cpu.pending_interrupt
            } else {
                trap::has_pending_interrupt(cpu)
            };

            if int.is_some() {
                let int = int.unwrap();

                trap::handle_interrupt(int, cpu);

                if cpu.pending_interrupt_number as u64 != 0 {
                    bus::get_bus()
                        .get_plic()
                        .update_pending(cpu.pending_interrupt_number as u64);
                }
            }
        }

        cpu.current_gpfn = cpu.next_pc >> RV_PAGE_SHIFT as CpuReg;
        cpu.current_guest_page = cpu.next_pc & RV_PAGE_MASK as CpuReg;

        let bus = bus::get_bus();

        let next_phys_pc = bus.translate(cpu.next_pc, &cpu.mmu, AccessType::Fetch);

        let next_phys_pc = if next_phys_pc.is_err() {
            cpu.exception = next_phys_pc.err().unwrap();
            cpu.c_exception_pc = cpu.next_pc as usize;
            trap::handle_exception(cpu);

            cpu.current_gpfn = cpu.next_pc >> RV_PAGE_SHIFT as CpuReg;
            cpu.current_guest_page = cpu.next_pc & RV_PAGE_MASK as CpuReg;

            bus.translate(cpu.next_pc, &cpu.mmu, AccessType::Fetch)
                .expect("Failed to translate pc after exception")
        } else {
            next_phys_pc.unwrap()
        };

        let mut insn_data = cpu.insn_map.get_by_guest_idx(next_phys_pc);
        if insn_data.is_none() {
            self.parse_core.parse_gpfn(None).unwrap();

            insn_data = cpu.insn_map.get_by_guest_idx(next_phys_pc);
        }

        insn_data.unwrap().host_ptr
    }

    pub fn exec_loop(&mut self, core_id: CpuReg, initial_pc: CpuReg) {
        let cpu = cpu::get_cpu();
        cpu.core_id = core_id;
        cpu.next_pc = initial_pc;

        cpu.regs[RegName::A0 as usize] = core_id;
        cpu.regs[RegName::A1 as usize] = DTB_BEGIN_ADDR;

        loop {
            let host_ptr = self.get_jit_ptr();

            cpu.exception = cpu::Exception::None;
            cpu.c_exception = cpu::Exception::None.to_cpu_reg() as usize;
            cpu.c_exception_data = 0;
            cpu.c_exception_pc = 0;
            cpu.jump_count = 0;
            cpu.next_pc = 0;

            let ret = if cpu.mmu.is_active() {
                ReturnableImpl::handle(|| unsafe {
                    BackendCoreImpl::call_jit_ptr(host_ptr);
                })
            } else {
                ReturnableImpl::handle(|| unsafe {
                    BackendCoreImpl::call_jit_ptr_nommu(host_ptr);
                })
            };

            match ret.return_status {
                ReturnStatus::ReturnOk => {
                    if cpu.c_exception == cpu::Exception::None.to_cpu_reg() as usize
                        && cpu
                            .has_pending_interrupt
                            .load(std::sync::atomic::Ordering::Acquire)
                            == 1
                    {
                        cpu.c_exception = cpu::Exception::None.to_cpu_reg() as usize;
                        cpu.exception = cpu::Exception::BookkeepingRet;
                    }
                }
                ReturnStatus::ReturnAccessViolation => {
                    let mut guest_exception_pc: Option<&InsnMappingData> = None;
                    let likely_offset = BackendCoreImpl::fastmem_violation_likely_offset();
                    let likely_offset_lower = likely_offset - 16;
                    let likely_offset_upper = likely_offset + 16;

                    let addr = ret.exception_address as *mut u8;

                    for i in likely_offset_lower..likely_offset_upper {
                        let exc = cpu.insn_map.get_by_host_ptr(addr.wrapping_sub(i));

                        if exc.is_some() {
                            guest_exception_pc = Some(exc.unwrap());
                            break;
                        }
                    }

                    if guest_exception_pc.is_none() {
                        panic!(
                            "Failed to find guest pc for host ptr {:#x}",
                            ret.exception_address
                        );
                    }

                    let guest_exception_pc = guest_exception_pc.unwrap();
                    let jit_block_idx = guest_exception_pc.jit_block_idx;

                    self.parse_core
                        .mark_page_state(jit_block_idx, PageState::ReadWrite)
                        .unwrap();

                    let handling_type = BackendCoreImpl::patch_fastmem_violation(
                        guest_exception_pc.host_ptr as usize,
                        guest_exception_pc.guest_idx,
                    );

                    self.parse_core
                        .mark_page_state(jit_block_idx, PageState::ReadExecute)
                        .unwrap();

                    if handling_type == FastmemHandleType::Manual {
                        cpu.c_exception_pc =
                            (guest_exception_pc.guest_idx & RV_PAGE_OFFSET_MASK as CpuReg) as usize;
                    } else {
                        cpu.exception = cpu::Exception::FastmemViolation;
                        cpu.next_pc = guest_exception_pc.guest_idx & RV_PAGE_OFFSET_MASK as CpuReg;
                        cpu.next_pc += cpu.current_gpfn << RV_PAGE_SHIFT as CpuReg;
                    }
                }
                _ => {
                    panic!("Unhandled host exception during guest execution")
                }
            }

            self.handle_guest_exception();
        }
    }

    fn handle_guest_exception(&mut self) {
        let cpu = cpu::get_cpu();

        if cpu.c_exception != cpu::Exception::None.to_cpu_reg() as usize {
            cpu.exception = cpu::Exception::from_cpu_reg(
                cpu.c_exception as CpuReg,
                cpu.c_exception_data as CpuReg,
            );
        }

        cpu.c_exception_pc += (cpu.current_gpfn as usize) << RV_PAGE_SHIFT;

        match cpu.exception {
            cpu::Exception::MmuStateUpdate => {
                cpu.next_pc = cpu.c_exception_pc as CpuReg + INSN_SIZE as CpuReg;
            }
            cpu::Exception::BlockExit => {
                cpu.next_pc = cpu.c_exception_pc as CpuReg;
            }
            cpu::Exception::ForwardJumpFault(pc) => {
                // We'll enter here both on unmapped jumps and missaligned jumps
                // In the case of missaligned jumps, we'll forward the exception
                // to the trap handler

                if pc % INSN_SIZE as CpuReg != 0 {
                    cpu.exception = cpu::Exception::InstructionAddressMisaligned(pc);
                    println!("Forward jump forwarding as {:?}", cpu.exception);
                    trap::handle_exception(cpu);
                } else {
                    cpu.next_pc = pc;
                }
            }
            cpu::Exception::InvalidateJitBlock(gpfn, should_reparse) => {
                self.parse_core.invalidate(gpfn, should_reparse);
                cpu.next_pc = cpu.c_exception_pc as CpuReg + INSN_SIZE as CpuReg;
            }
            cpu::Exception::FastmemViolation => {}
            cpu::Exception::DiscardJitBlock(_pc) => {
                // If a mmu drops execute permission on a page, we can discard the jit block
                unimplemented!()
            }
            cpu::Exception::Wfi => {
                // I hate this from the bottom of my heart but the altrenative is making
                // all csr accesses atomic which is probably worse more so this will have to do
                if trap::are_interrupts_enabled(cpu) {
                    let bus = bus::get_bus();
                    std::thread::sleep(std::time::Duration::from_millis(
                        Clint::get_remaining_time_ms(),
                    ));

                    loop {
                        bus.tick_core_local();

                        if cpu
                            .has_pending_interrupt
                            .load(std::sync::atomic::Ordering::Acquire)
                            == 1
                        {
                            cpu.has_pending_interrupt
                                .store(0, std::sync::atomic::Ordering::Release);
                            cpu.pending_interrupt = None;
                            break;
                        }

                        std::thread::sleep(std::time::Duration::from_millis(50));
                    }
                }

                cpu.next_pc = cpu.c_exception_pc as CpuReg + INSN_SIZE as CpuReg;
            }
            cpu::Exception::BookkeepingRet => {
                // There may be a pending exception but that cannot checked unless we exit
                // the jit block. Epsecially for cases where infinite loops are used,
                // we need to make sure we periodiaclly exit the jit block to check
                // for interrupts
                cpu.next_pc = cpu.c_exception_pc as CpuReg;
            }
            cpu::Exception::Mret | cpu::Exception::Sret => {}
            cpu::Exception::None => {
                unreachable!("Exiting jit block without setting an exception is invalid");
            }
            _ => {
                trap::handle_exception(cpu);
                if matches!(
                    cpu.exception,
                    cpu::Exception::EnvironmentCallFromSMode(_)
                        | cpu::Exception::IllegalInstruction(_)
                ) {
                } else {
                    println!(
                        "ret_status: {:#x?} with pc 0x{:x} cpu.next_pc {:x} gp {}",
                        cpu.exception, cpu.c_exception_pc, cpu.next_pc, cpu.regs[3]
                    );
                }
            }
        }
    }
}

pub fn exec_core_thread(cpu_core_idx: usize, initial_pc: CpuReg) {
    cpu::init_cpu();

    let mut exec_core = ExecCore::new();

    let cpu = cpu::get_cpu() as *mut cpu::Cpu as usize;

    std::thread::spawn(move || {
        let bus = bus::get_bus();
        let cpu = unsafe { &mut *(cpu as *mut cpu::Cpu) };

        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));

            if let Some(irqn) = bus.tick_async(cpu) {
                if irqn != 0 {
                    cpu.csr
                        .write_bit(csr::register::MIP, csr::bits::SEIP_BIT, true);

                    cpu.pending_interrupt_number = irqn;
                }
            }

            trap::has_pending_interrupt(cpu);
        }
    });

    exec_core.exec_loop(cpu_core_idx as CpuReg, initial_pc);
}

pub struct ExecCoreThreadPool {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl ExecCoreThreadPool {
    pub fn new(ram_begin_addr: BusType, thread_count: usize) -> Self {
        let mut threads = Vec::new();

        for core_id in 0..thread_count {
            threads.push(std::thread::spawn(move || {
                exec_core_thread(core_id, ram_begin_addr)
            }));
        }

        Self { threads }
    }

    pub fn join(self) {
        for thread in self.threads {
            thread.join().unwrap();
        }
    }
}
