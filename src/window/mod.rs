pub mod console;
pub mod window;

#[cfg(windows)]
pub use console::win32_console_settings as ConsoleSettings;

#[cfg(unix)]
pub use console::posix_console_settings as ConsoleSettings;
