pub mod console;

#[cfg(windows)]
pub use console::win32_console_settings as ConsoleSettings;
