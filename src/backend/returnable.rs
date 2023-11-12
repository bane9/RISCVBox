pub use crate::backend::ReturnableImpl;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnStatus {
    ReturnOk,
    ReturnNotify,
}

pub trait ReturnableHandler {
    #[must_use]
    fn handle<F: Fn() -> ()>(closure: F) -> ReturnStatus;
    fn throw() -> !;
}

pub extern "C" fn c_return_ok() {
    ReturnableImpl::throw()
}
