pub use crate::backend::ReturnableImpl;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnStatus {
    ReturnOk,
    ReturnNotify,
}

pub trait ReturnableHandler {
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
