pub mod sdl2_dark_mode;
pub mod sdl2_wnd;
pub mod window_common;

pub use self::window_common::WindowCommon;

pub use sdl2_wnd::Sdl2Window as window_impl;

#[cfg(windows)]
pub use sdl2_dark_mode::sdl2_dark_mode_win::enable_dark_mode_for_window;

#[cfg(not(windows))]
pub use sdl2_dark_mode::sdl2_dark_mode_nop::enable_dark_mode_for_window_nop as enable_dark_mode_for_window;
