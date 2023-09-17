#[cfg(target_arch = "x86_64")]
pub mod amd64;

#[cfg(target_arch = "x86_64")]
pub use amd64 as target;

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64 as target;

pub use target::*;

pub mod setjmp_common;

#[cfg(unix)]
pub mod setjmp_posix;

pub mod common;
pub use common::*;
