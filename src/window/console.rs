#[cfg(windows)]
pub mod win32_console_settings {
    extern crate winapi;

    use winapi::um::fileapi::GetFileType;
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::GetStdHandle;
    use winapi::um::wincon::{ENABLE_ECHO_INPUT, ENABLE_LINE_INPUT, ENABLE_PROCESSED_INPUT};

    static mut OLD_CONSOLE_MODE: Option<winapi::shared::minwindef::DWORD> = None;

    extern "C" {
        #[link_name = "SetConsoleMode"]
        fn SetConsoleMode(
            handle: winapi::um::winnt::HANDLE,
            mode: winapi::shared::minwindef::DWORD,
        ) -> winapi::shared::minwindef::BOOL;
        #[link_name = "GetConsoleMode"]
        fn GetConsoleMode(
            handle: winapi::um::winnt::HANDLE,
            mode: *mut winapi::shared::minwindef::DWORD,
        ) -> winapi::shared::minwindef::BOOL;
        #[link_name = "atexit"]
        fn atexit(callback: unsafe extern "C" fn());
    }

    unsafe extern "C" fn set_old_console_mode() {
        if let Some(mode) = OLD_CONSOLE_MODE {
            let handle = GetStdHandle(winapi::um::winbase::STD_INPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE {
                eprintln!("Failed to get console handle");
                return;
            }

            if SetConsoleMode(handle, mode) == 0 {
                eprintln!("Failed to set console mode");
            }
        }
    }

    pub fn set_interactive_console() {
        unsafe {
            let handle = GetStdHandle(winapi::um::winbase::STD_INPUT_HANDLE);
            if handle == INVALID_HANDLE_VALUE || GetFileType(handle) != 0x0002 {
                eprintln!("Failed to get console handle");
                return;
            }

            let mut mode: winapi::shared::minwindef::DWORD = 0;
            if GetConsoleMode(handle, &mut mode) == 0 {
                eprintln!("Failed to get console mode");
                return;
            }

            if OLD_CONSOLE_MODE.is_none() {
                OLD_CONSOLE_MODE = Some(mode);
            }

            let new_mode = mode & !(ENABLE_LINE_INPUT | ENABLE_ECHO_INPUT | ENABLE_PROCESSED_INPUT);

            if SetConsoleMode(handle, new_mode) == 0 {
                eprintln!("Failed to set console mode");
            }

            atexit(set_old_console_mode);
        }
    }
}

#[cfg(unix)]
pub mod posix_console_settings {
    extern crate termios;

    use termios::{tcsetattr, Termios, TCSAFLUSH};

    static mut OLD_TERMINAL_MODE: Option<Termios> = None;

    extern "C" fn set_old_terminal_mode() {
        unsafe {
            if let Some(termios) = OLD_TERMINAL_MODE {
                tcsetattr(0, TCSAFLUSH, &termios).unwrap();
            }
        }
    }

    pub fn set_interactive_console() {
        let mut termios = Termios::from_fd(0).unwrap();

        if let None = unsafe { OLD_TERMINAL_MODE } {
            unsafe {
                OLD_TERMINAL_MODE = Some(termios);
            }
        }

        termios.c_lflag &= !(termios::ECHO | termios::ICANON | termios::ISIG | termios::IEXTEN);

        termios.c_iflag &= !(termios::IXON | termios::ICRNL);

        tcsetattr(0, TCSAFLUSH, &termios).unwrap();

        unsafe {
            libc::atexit(set_old_terminal_mode);
        }
    }
}
