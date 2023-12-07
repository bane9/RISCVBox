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
            microseh::ExceptionCode::IllegalInstruction => ReturnStatus::ReturnOk,
            microseh::ExceptionCode::AccessViolation => {
                let addr = res.err().unwrap().address() as usize;
                ReturnStatus::ReturnAccessViolation(addr)
            }
            code => {
                println!("code: {:?}", code);
                ReturnStatus::ReturnNotOk
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
