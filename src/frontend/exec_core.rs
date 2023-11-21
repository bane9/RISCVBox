use crate::backend::target::core::BackendCoreImpl;
use crate::backend::{BackendCore, ReturnStatus, ReturnableHandler, ReturnableImpl};
use crate::bus::mmu::AccessType;
use crate::bus::{self, BusType};
use crate::cpu::trap;
use crate::cpu::{self, CpuReg};
pub use crate::frontend::parse_core::*;

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

        let next_phys_pc = bus::get_bus()
            .translate(cpu.next_pc, &cpu.mmu, AccessType::Fetch)
            .unwrap();

        let mut insn_data = cpu.insn_map.get_by_guest_idx(next_phys_pc);
        if insn_data.is_none() {
            self.parse_core.parse_gpfn(None).unwrap();

            insn_data = cpu.insn_map.get_by_guest_idx(next_phys_pc);
        }

        cpu.current_gpfn = cpu.next_pc >> RV_PAGE_SHIFT as CpuReg;

        insn_data.unwrap().host_ptr
    }

    pub fn exec_loop(&mut self, initial_pc: CpuReg) {
        let cpu = cpu::get_cpu();
        cpu.next_pc = initial_pc;

        loop {
            let host_ptr = self.get_jit_ptr();

            cpu.exception = cpu::Exception::None;
            cpu.c_exception = cpu::Exception::None.to_cpu_reg() as usize;
            cpu.c_exception_data = 0;
            cpu.c_exception_pc = 0;

            // unsafe {
            //     BackendCoreImpl::call_jit_ptr(insn_data.unwrap().host_ptr);
            // }

            let ret = ReturnableImpl::handle(|| unsafe {
                BackendCoreImpl::call_jit_ptr(host_ptr);
            });

            assert!(ret == ReturnStatus::ReturnOk);

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

        cpu.c_exception_pc |= (cpu.current_gpfn as usize) << RV_PAGE_SHIFT;

        println!(
            "ret_status: {:#x?} with pc 0x{:x} cpu.next_pc {:x} gp {}",
            cpu.exception, cpu.c_exception_pc, cpu.next_pc, cpu.regs[3]
        );

        match cpu.exception {
            cpu::Exception::MmuStateUpdate | cpu::Exception::BlockExit => {
                cpu.next_pc = cpu.c_exception_pc as CpuReg + INSN_SIZE as CpuReg;
            }
            cpu::Exception::ForwardJumpFault(pc) => {
                // We'll enter here both on unmapped jumps and missaligned jumps
                // In the case of missaligned jumps, we'll forward the exception
                // to the trap handler

                if pc % INSN_SIZE as CpuReg != 0 {
                    cpu.exception = cpu::Exception::InstructionAddressMisaligned(pc);
                    println!("Forward jump forwarding as {:?}", cpu.exception);
                    trap::handle_exception();
                } else {
                    cpu.next_pc = cpu.c_exception_pc as CpuReg;
                }
            }
            cpu::Exception::InvalidateJitBlock(gpfn) => {
                self.parse_core.invalidate(gpfn);
                cpu.next_pc = cpu.c_exception_pc as CpuReg + INSN_SIZE as CpuReg;
            }
            cpu::Exception::DiscardJitBlock(_pc) => {
                // If a mmu drops execute permission on a page, we can discard the jit block
                unimplemented!()
            }
            cpu::Exception::Mret | cpu::Exception::Sret => {}
            cpu::Exception::None => {
                unreachable!("Exiting jit block without setting an exception is invalid");
            }
            _ => {
                trap::handle_exception();
            }
        }
    }
}

pub fn exec_core_thread(initial_pc: CpuReg) {
    let mut exec_core = ExecCore::new();

    exec_core.exec_loop(initial_pc);
}

pub struct ExecCoreThreadPool {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl ExecCoreThreadPool {
    pub fn new(ram_begin_addr: BusType, thread_count: usize) -> Self {
        let mut threads = Vec::new();

        for _ in 0..thread_count {
            threads.push(std::thread::spawn(move || exec_core_thread(ram_begin_addr)));
        }

        Self { threads }
    }

    pub fn join(self) {
        for thread in self.threads {
            thread.join().unwrap();
        }
    }
}
