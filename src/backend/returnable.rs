pub use crate::backend::ReturnableImpl;

pub type ReturnableClosure = fn();

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnStatus {
    ReturnOk,
    ReturnNotify,
}

pub trait ReturnableHandler {
    fn handle(closure: ReturnableClosure) -> ReturnStatus;
    fn return_ok() -> !;
    fn return_notify() -> !;
}

pub extern "C" fn c_return_ok() {
    ReturnableImpl::return_ok()
}

pub extern "C" fn c_return_notify() {
    ReturnableImpl::return_notify()
}
