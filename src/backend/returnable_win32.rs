// This is absolutely horrible but at least it works on windows unlike setjmp

pub use crate::backend::returnable::*;

use microseh;

pub struct ReturnableImpl;
use std::arch::asm;

impl ReturnableHandler for ReturnableImpl {
    fn handle<F: Fn() -> ()>(closure: F) -> ReturnStatus {
        let res = microseh::try_seh(closure);

        if let Ok(_) = res {
            return ReturnStatus::ReturnOk;
        }

        match res.err().unwrap().code() {
            microseh::ExceptionCode::Breakpoint => ReturnStatus::ReturnOk,
            microseh::ExceptionCode::IntDivideByZero => ReturnStatus::ReturnNotify,
            _ => panic!("unknown return status"),
        }
    }

    #[cfg(target_arch = "x86_64")]
    fn throw() -> ! {
        unsafe { asm!("int3") };

        unreachable!();
    }

    #[cfg(target_arch = "aarch64")]
    fn throw() -> ! {
        unsafe { asm!("brk 0;") };

        unreachable!();
    }
}
