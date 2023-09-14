#[cfg(target_arch = "x86_64")]
pub mod amd64;

#[cfg(target_arch = "x86_64")]
pub use amd64 as target;

pub use target::*;

pub mod common;
pub use common::*;
