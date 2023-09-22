use crate::backend::{ReturnableHandler, ReturnableImpl};
use crate::cpu;
pub use crate::frontend::parse_core::*;

pub struct ExecCore {
    parse_core: ParseCore,
}

impl ExecCore {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            parse_core: ParseCore::new(rom),
        }
    }

    pub fn exec_loop(&mut self) {
        let ptr = self.parse_core.get_exec_ptr();
        let cpu = cpu::get_cpu();
        cpu.ret_status = cpu::RunState::Running as usize;
        loop {
            let callable: extern "C" fn() = unsafe { std::mem::transmute(ptr) };
            //let result = ReturnableImpl::handle(|| callable());

            callable();
            //println!("result: {:?}", result);
            println!(
                "ret_status: {:?}",
                cpu::RunState::from_usize(cpu.ret_status)
            );
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
