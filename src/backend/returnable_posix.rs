pub use crate::backend::returnable::*;
use std::cell::RefCell;
use std::ffi::{c_int, c_void};

extern crate libc;
use libc::{exit, sigaction, siginfo_t};

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

extern "C" {
    fn setjmp(env: *mut c_void) -> i32;
    fn longjmp(env: *mut c_void, val: c_int) -> !;
}

const JUMP_BUF_SIZE: usize = 256;
type JmpBuf = [u8; JUMP_BUF_SIZE];

extern "C" fn sigaction_handler(signum: c_int, _info: *mut siginfo_t, context: *mut c_void) {
    let in_jit_block = IN_JIT_BLOCK.with(|in_jit_block| *in_jit_block.borrow());

    if !in_jit_block {
        println!("Signal received outside of JIT block: {}", signum);
        unsafe { exit(1) };
    }

    let jmp_buf = JUMP_BUF.with(|buf| (*buf.borrow()).as_ptr());

    let jmp_sig = match signum {
        11 => RETURN_ACCESS_VIOLATION,
        _ => RETURN_NOT_OK,
    };

    unsafe {
        let context = context as *const libc::ucontext_t;

        let host_regs = (*context).uc_mcontext.gregs;

        let regs = Registers::new_from_slice(&[
            host_regs[libc::REG_RAX as usize] as usize,
            host_regs[libc::REG_RCX as usize] as usize,
            host_regs[libc::REG_RDX as usize] as usize,
            host_regs[libc::REG_RBX as usize] as usize,
            host_regs[libc::REG_RSP as usize] as usize,
            host_regs[libc::REG_RBP as usize] as usize,
            host_regs[libc::REG_RSI as usize] as usize,
            host_regs[libc::REG_RDI as usize] as usize,
            host_regs[libc::REG_R8 as usize] as usize,
            host_regs[libc::REG_R9 as usize] as usize,
            host_regs[libc::REG_R10 as usize] as usize,
            host_regs[libc::REG_R11 as usize] as usize,
            host_regs[libc::REG_R12 as usize] as usize,
            host_regs[libc::REG_R13 as usize] as usize,
            host_regs[libc::REG_R14 as usize] as usize,
            host_regs[libc::REG_R15 as usize] as usize,
            host_regs[libc::REG_RIP as usize] as usize,
        ]);

        REGISTERS.with(|regs_cell| {
            *regs_cell.borrow_mut() = regs;
        });

        EXCEPTION_ADDR
            .with(|addr_cell| *addr_cell.borrow_mut() = host_regs[libc::REG_RIP as usize] as usize);
    }

    unsafe {
        longjmp(jmp_buf as *mut c_void, jmp_sig);
    }
}

thread_local! {
    static JUMP_BUF: RefCell<Box<JmpBuf>> = RefCell::new(Box::new([0; JUMP_BUF_SIZE]));
    static REGISTERS: RefCell<Registers> = RefCell::new(Registers::new());
    static EXCEPTION_ADDR: RefCell<usize> = RefCell::new(0);
    static IN_JIT_BLOCK: RefCell<bool> = RefCell::new(false);
}

const RETURN_FIRST_CALL: i32 = 0;
const RETURN_OK: i32 = 1;
const RETURN_ACCESS_VIOLATION: i32 = 2;
const RETURN_NOT_OK: i32 = 3;

pub struct ReturnableImpl;

fn setup_sigaction() {
    static mut ONCE: bool = false;

    if unsafe { ONCE } {
        return;
    }

    let mut sa: sigaction = unsafe { std::mem::zeroed() };
    sa.sa_sigaction = sigaction_handler as usize;

    for i in 1..64 {
        unsafe {
            sigaction(i, &sa, std::ptr::null_mut());
        }
    }

    unsafe {
        ONCE = true;
    }
}

impl ReturnableHandler for ReturnableImpl {
    fn handle<F: Fn() -> ()>(closure: F) -> ExceptionInfo {
        setup_sigaction();

        let jmp_buf = JUMP_BUF.with(|buf| (*buf.borrow()).as_ptr());

        let i = unsafe { setjmp(jmp_buf as *mut c_void) };

        IN_JIT_BLOCK.with(|in_jit_block| {
            *in_jit_block.borrow_mut() = true;
        });

        if i == RETURN_FIRST_CALL {
            closure();
        }

        IN_JIT_BLOCK.with(|in_jit_block| {
            *in_jit_block.borrow_mut() = false;
        });

        let ret_code = match i {
            RETURN_FIRST_CALL | RETURN_OK => ReturnStatus::ReturnOk,
            RETURN_ACCESS_VIOLATION => ReturnStatus::ReturnAccessViolation,
            RETURN_NOT_OK => ReturnStatus::ReturnNotOk,
            _ => panic!("Unknown return status: {}", i),
        };

        let regs = REGISTERS.with(|regs_cell| regs_cell.borrow().regs);
        let addr = EXCEPTION_ADDR.with(|addr_cell| *addr_cell.borrow());

        ExceptionInfo::new_from_silce(ret_code, addr, &regs)
    }

    #[allow(unreachable_code)]
    fn throw() -> ! {
        let jmp_buf = JUMP_BUF.with(|buf| (*buf.borrow()).as_ptr());

        unsafe {
            longjmp(jmp_buf as *mut c_void, RETURN_OK);
        }

        unreachable!();
    }
}
