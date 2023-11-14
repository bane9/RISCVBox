pub trait CodePage {
    fn new() -> Self;

    fn push(&mut self, data: &[u8]) -> Result<(), AllocationError>;

    fn mark_rw(&mut self) -> Result<(), AllocationError>;

    fn mark_rx(&mut self) -> Result<(), AllocationError>;

    fn mark_invalid(&mut self) -> Result<(), AllocationError>;

    fn dealloc(&mut self);

    fn as_ptr(&self) -> *mut u8;
    fn as_end_ptr(&self) -> *mut u8 {
        unsafe { self.as_ptr().add(self.size()) }
    }

    fn size(&self) -> usize;
    fn npages(&self) -> usize;

    fn state(&self) -> PageState;
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum AllocationError {
    UnknownError,
    OutOfMemory,
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum PageState {
    ReadWrite,
    ReadExecute,
    Invalid,
}
