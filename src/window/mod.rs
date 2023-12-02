pub mod sdl2;
pub mod window_common;

pub use self::window_common::WindowCommon;

pub use sdl2::Sdl2Window as window_impl;
