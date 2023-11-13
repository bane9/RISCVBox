use crate::backend::target::core::BackendCoreImpl;
use crate::backend::BackendCore;
use crate::cpu::{self, CpuReg};
pub use crate::frontend::parse_core::*;

pub struct ExecCore {
    parse_core: ParseCore,
}

impl ExecCore {
    pub fn new(rom: Vec<u8>) -> Self {
        let mut parse_core = ParseCore::new(rom.len());
        parse_core.parse_ahead().unwrap();
        Self { parse_core }
    }

    pub fn exec_loop(&mut self) {
        let ptr = self.parse_core.get_exec_ptr();
        let cpu = cpu::get_cpu();
        loop {
            // let result = ReturnableImpl::handle(|| unsafe { BackendCoreImpl::call_jit_ptr(ptr) });

            cpu.exception = cpu::Exception::None;
            cpu.c_exception = cpu::Exception::None.to_cpu_reg() as usize;
            cpu.c_exception_data = 0;

            unsafe {
                BackendCoreImpl::call_jit_ptr(ptr);
            }

            if cpu.c_exception != cpu::Exception::None.to_cpu_reg() as usize {
                cpu.exception = cpu::Exception::from_cpu_reg(
                    cpu.c_exception as CpuReg,
                    cpu.c_exception_data as CpuReg,
                );
            }

            println!("ret_status: {:#x?}", cpu.exception);
            break;
        }
    }
}

pub fn exec_core_thread(rom: Vec<u8>) {
    let mut exec_core = ExecCore::new(rom);

    exec_core.exec_loop();
}

pub struct ExecCoreThreadPool {
    threads: Vec<std::thread::JoinHandle<()>>,
}

impl ExecCoreThreadPool {
    pub fn new(rom: Vec<u8>, thread_count: usize) -> Self {
        let mut threads = Vec::new();

        for _ in 0..thread_count {
            let rom_local = rom.clone();
            threads.push(std::thread::spawn(move || exec_core_thread(rom_local)));
        }

        Self { threads }
    }

    pub fn join(self) {
        for thread in self.threads {
            thread.join().unwrap();
        }
    }
}
