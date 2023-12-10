#[cfg(all(target_arch = "x86_64"))]
pub mod amd64;

#[cfg(all(target_arch = "x86_64"))]
pub use amd64 as target;

#[cfg(all(target_arch = "aarch64"))]
pub mod aarch64;

#[cfg(all(target_arch = "aarch64"))]
pub use aarch64 as target;

pub use target::*;

pub mod returnable;
pub use returnable::*;

#[cfg(unix)]
pub mod returnable_posix;

#[cfg(unix)]
pub use returnable_posix::ReturnableImpl;

#[cfg(windows)]
pub mod returnable_win32;

#[cfg(windows)]
pub use returnable_win32::ReturnableImpl;

pub mod common;
pub use common::*;

pub mod csr;
pub use csr::CsrImpl;

pub mod rva;
pub use rva::RvaImpl;
