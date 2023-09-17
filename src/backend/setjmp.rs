// Insanely UB, use with care

pub use std::cell::RefCell;
pub use std::ffi::{c_int, c_void};
pub use std::rc::Rc;

extern "C" {
    pub fn setjmp(env: *mut c_void) -> i32;
    pub fn longjmp(env: *mut c_void, val: c_int);
}

pub const JUMP_BUF_SIZE: usize = 256;
pub type JmpBuf = [u8; JUMP_BUF_SIZE];

#[macro_export]
macro_rules! declare_setjmp_buffer {
    ($buffer_name:ident) => {
        thread_local! {
            static $buffer_name: RefCell<Box<JmpBuf>> = RefCell::new(Box::new([0; JUMP_BUF_SIZE]));
        }
    };
}

#[macro_export]
macro_rules! get_jmp_buf {
    ($buffer_name:ident) => {
        $buffer_name.with(|buf| (*buf.borrow()).as_ptr());
    };
}
