use crate::backend::common::BackendCore;
use crate::backend::target::core::BackendCoreImpl;
pub use crate::backend::ReturnableImpl;
use crate::host_get_return_addr;
use std::arch::asm;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnStatus {
    ReturnOk,
    ReturnNotify,
}

pub trait ReturnableHandler {
    #[must_use]
    fn handle<F: Fn() -> ()>(closure: F) -> ReturnStatus;
    fn return_ok() -> !;
    fn return_notify() -> !;
}

pub extern "C" fn c_return_ok() {
    ReturnableImpl::return_ok()
}

pub extern "C" fn c_return_notify() {
    ReturnableImpl::return_notify()
}
