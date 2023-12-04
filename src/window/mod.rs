pub mod sdl2_wnd;
pub mod window_common;

pub use self::window_common::WindowCommon;

pub use sdl2_wnd::Sdl2Window as window_impl;
