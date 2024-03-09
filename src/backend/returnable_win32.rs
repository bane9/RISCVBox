pub use crate::backend::returnable::*;

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Register {
    Rax = 0,
    Rcx = 1,
    Rdx = 2,
    Rbx = 3,
    Rsp = 4,
    Rbp = 5,
    Rsi = 6,
    Rdi = 7,
    R8 = 8,
    R9 = 9,
    R10 = 10,
    R11 = 11,
    R12 = 12,
    R13 = 13,
    R14 = 14,
    R15 = 15,

    Rip = 16,

    Count = 17,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Registers {
    pub regs: [usize; Register::Count as usize],
}

impl Registers {
    pub fn new() -> Registers {
        Registers {
            regs: [0; Register::Count as usize],
        }
    }

    pub fn new_from_slice(slice: &[usize]) -> Registers {
        let mut regs = [0; Register::Count as usize];

        for (i, reg) in slice.iter().enumerate() {
            regs[i] = *reg;
        }

        Registers { regs }
    }
}

impl ReturnableRegisterData for Registers {
    fn get_register_data(&self, register: Register) -> usize {
        self.regs[register as usize]
    }

    fn set_register_data(&mut self, register: Register, value: usize) {
        self.regs[register as usize] = value;
    }
}

pub struct ReturnableImpl;
use std::arch::asm;

impl ReturnableHandler for ReturnableImpl {
    fn handle<F: Fn() -> ()>(closure: F) -> ExceptionInfo {
        let res = microseh::try_seh(closure);

        if let Ok(_) = res {
            return ExceptionInfo::new_from_silce(ReturnStatus::ReturnOk, 0, &[0; 17]);
        }

        let err = res.err().unwrap();
        let addr = err.address() as usize;

        match err.code() {
            microseh::ExceptionCode::IllegalInstruction => {
                ExceptionInfo::new_from_silce(ReturnStatus::ReturnOk, addr, err.registers().list())
            }
            microseh::ExceptionCode::AccessViolation => ExceptionInfo::new_from_silce(
                ReturnStatus::ReturnAccessViolation,
                addr,
                err.registers().list(),
            ),
            code => {
                println!("code: {:?}", code);
                ExceptionInfo::new_from_silce(
                    ReturnStatus::ReturnNotOk,
                    addr,
                    err.registers().list(),
                )
            }
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn throw() -> ! {
        unsafe { asm!("ud2") };

        unreachable!();
    }

    #[cfg(target_arch = "aarch64")]
    fn throw() -> ! {
        unsafe { asm!("brk 0;") };

        unreachable!();
    }
}
