mod page_common;

pub use page_common::*;

#[cfg(all(windows, not(feature = "nojit")))]
mod page_win32;

#[cfg(all(unix, not(feature = "nojit")))]
mod page_posix;

#[cfg(all(windows, not(feature = "nojit")))]
pub use crate::xmem::page_win32::Win32Allocator as CodePageImpl;

#[cfg(all(unix, not(feature = "nojit")))]
pub use crate::xmem::page_posix::PosixAllocator as CodePageImpl;
