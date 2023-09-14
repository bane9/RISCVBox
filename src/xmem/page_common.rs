pub trait PageAllocator {
    fn alloc(npages: usize) -> Result<*mut u8, AllocationError>;

    fn realloc(
        ptr: *mut u8,
        old_npages: usize,
        new_npages: usize,
    ) -> Result<*mut u8, AllocationError>;

    fn mark_rw(ptr: *mut u8, npages: usize) -> Result<(), AllocationError>;

    fn mark_rx(ptr: *mut u8, npages: usize) -> Result<(), AllocationError>;

    fn dealloc(ptr: *mut u8, npages: usize);

    fn page_size() -> usize;
}

#[derive(Debug)]
pub enum AllocationError {
    UnknownError,
    OutOfMemory,
}
