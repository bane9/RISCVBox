use crate::backend::target::core::BackendCoreImpl;
use crate::backend::{BackendCore, ReturnStatus, ReturnableHandler, ReturnableImpl};
use crate::bus::BusType;
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

    pub fn exec_loop(&mut self, initial_pc: CpuReg) {
        let cpu = cpu::get_cpu();
        cpu.pc = initial_pc;

        loop {
            let mut insn_data = cpu.insn_map.get_by_guest_idx(cpu.pc);
            if insn_data.is_none() {
                let pc = cpu.pc;

                self.parse_core
                    .parse(cpu.pc as usize, cpu.pc as usize + INSN_PAGE_SIZE as usize)
                    .unwrap();

                cpu.pc = pc;

                insn_data = cpu.insn_map.get_by_guest_idx(cpu.pc);
            }

            cpu.exception = cpu::Exception::None;
            cpu.c_exception = cpu::Exception::None.to_cpu_reg() as usize;
            cpu.c_exception_data = 0;
            cpu.c_exception_pc = 0;

            // unsafe {
            //     BackendCoreImpl::call_jit_ptr(insn_data.unwrap().host_ptr);
            // }

            let ret = ReturnableImpl::handle(|| unsafe {
                let host_ptr = insn_data.unwrap().host_ptr;
                BackendCoreImpl::call_jit_ptr(host_ptr);
            });

            assert!(ret == ReturnStatus::ReturnOk);

            if cpu.c_exception != cpu::Exception::None.to_cpu_reg() as usize {
                cpu.exception = cpu::Exception::from_cpu_reg(
                    cpu.c_exception as CpuReg,
                    cpu.c_exception_data as CpuReg,
                );
            }

            println!(
                "ret_status: {:#x?} with pc 0x{:x}",
                cpu.exception, cpu.c_exception_pc
            );

            match cpu.exception {
                cpu::Exception::BlockExit => {
                    cpu.pc = cpu.c_exception_pc as CpuReg + INSN_SIZE as CpuReg;
                }
                cpu::Exception::None => {
                    unreachable!("Exiting jit block without exception is invalid");
                }
                cpu::Exception::ForwardJumpFault(pc) => {
                    println!("ForwardJumpFault: pc = {:#x}", pc);
                    std::process::exit(1);
                }
                cpu::Exception::IllegalInstruction(pc) => {
                    println!("IllegalInstruction: pc = {:#x}", pc);
                    std::process::exit(1);
                }
                cpu::Exception::Mret | cpu::Exception::Sret => {}
                _ => {
                    trap::handle_exception();
                }
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
