pub use crate::backend::{Register, Registers};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnStatus {
    ReturnOk,
    ReturnAccessViolation,
    ReturnNotOk,
}

pub trait ReturnableRegisterData {
    fn get_register_data(&self, register: Register) -> usize;
    fn set_register_data(&mut self, register: Register, value: usize);
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ExceptionInfo {
    pub return_status: ReturnStatus,
    pub exception_address: usize,
    pub registers: Registers,
}

impl ExceptionInfo {
    pub fn new(
        return_status: ReturnStatus,
        exception_address: usize,
        registers: Registers,
    ) -> ExceptionInfo {
        ExceptionInfo {
            return_status,
            exception_address,
            registers,
        }
    }

    pub fn new_from_silce(
        return_status: ReturnStatus,
        exception_address: usize,
        slice: &[usize],
    ) -> ExceptionInfo {
        ExceptionInfo {
            return_status,
            exception_address,
            registers: Registers::new_from_slice(slice),
        }
    }
}

pub trait ReturnableHandler {
    #[must_use]
    fn handle<F: Fn() -> ()>(closure: F) -> ExceptionInfo;
    fn throw() -> !;
}
