#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ReturnStatus {
    ReturnOk,
    ReturnNotOk,
}

pub trait ReturnableHandler {
    #[must_use]
    fn handle<F: Fn() -> ()>(closure: F) -> ReturnStatus;
    fn throw() -> !;
}
