// Insanely UB, use with care

pub use crate::backend::returnable::*;
use std::cell::RefCell;
use std::ffi::{c_int, c_void};

extern "C" {
    fn setjmp(env: *mut c_void) -> i32;
    fn longjmp(env: *mut c_void, val: c_int) -> !;
}

const JUMP_BUF_SIZE: usize = 256;
type JmpBuf = [u8; JUMP_BUF_SIZE];

thread_local! {
    static JUMP_BUF: RefCell<Box<JmpBuf>> = RefCell::new(Box::new([0; JUMP_BUF_SIZE]));
}

pub struct ReturnableImpl;

impl ReturnableHandler for ReturnableImpl {
    fn handle<F: Fn() -> ()>(closure: F) -> ReturnStatus {
        let jmp_buf = JUMP_BUF.with(|buf| (*buf.borrow()).as_ptr());

        let i = unsafe { setjmp(jmp_buf as *mut c_void) };

        if i == 0 {
            closure();
        }

        match i {
            1 => ReturnStatus::ReturnOk,
            2 => ReturnStatus::ReturnNotify,
            _ => panic!("unknown return status"),
        }
    }

    fn return_ok() -> ! {
        let jmp_buf = JUMP_BUF.with(|buf| (*buf.borrow()).as_ptr());

        unsafe {
            longjmp(jmp_buf as *mut c_void, 1);
        }
    }

    fn return_notify() -> ! {
        let jmp_buf = JUMP_BUF.with(|buf| (*buf.borrow()).as_ptr());

        unsafe {
            longjmp(jmp_buf as *mut c_void, 2);
        }
    }
}
