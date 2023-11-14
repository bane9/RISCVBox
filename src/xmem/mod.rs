mod page_common;

pub use page_common::*;

#[cfg(target_os = "windows")]
mod page_win32;

#[cfg(unix)]
mod page_posix;

#[cfg(target_os = "windows")]
pub use crate::xmem::page_win32::Win32Allocator as CodePageImpl;

#[cfg(unix)]
pub use crate::xmem::page_posix::PosixAllocator as CodePageImpl;
