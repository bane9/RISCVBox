pub mod console;
pub mod window;

#[cfg(windows)]
pub use console::win32_console_settings as ConsoleSettings;
