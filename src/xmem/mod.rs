mod page_allocator;
mod page_common;

pub use page_common::*;

#[cfg(all(windows))]
pub use crate::xmem::page_allocator::win32_page_allocator as PageAllocator;

#[cfg(all(unix))]
pub use crate::xmem::page_allocator::posix_page_allocator as PageAllocator;
