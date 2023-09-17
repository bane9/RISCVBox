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
