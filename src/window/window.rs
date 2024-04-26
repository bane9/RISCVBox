use crate::bus::ns16550::{write_char_cb, write_char_kbd};
use minifb::{self, Key};

struct UartCB;

impl minifb::InputCallback for UartCB {
    fn add_char(&mut self, c: u32) {
        write_char_cb(c as u8);
    }

    fn set_key_state(&mut self, key: Key, state: bool) {
        if key >= Key::A && key <= Key::Z || key == Key::Space {
            let mut key = key as u32;

            if key == Key::Space as u32 {
                key = b' ' as u32;
            } else {
                key = key - Key::A as u32 + b'a' as u32;
            }

            key = if state { key } else { key | 0x80 };

            write_char_kbd(key as u8);
        }
    }
}

pub struct Window {
    window: minifb::Window,
    width: usize,
    height: usize,
    fb_slice: &'static [u32],
    framebuffer: Vec<u32>,
}

impl Window {
    pub fn new(fb_ptr: *mut u8, width: usize, height: usize, scale: usize) -> Self {
        let mut options = minifb::WindowOptions::default();

        options.scale = match scale {
            1 => minifb::Scale::X1,
            2 => minifb::Scale::X2,
            4 => minifb::Scale::X4,
            8 => minifb::Scale::X8,
            16 => minifb::Scale::X16,
            32 => minifb::Scale::X32,
            _ => panic!("Invalid framebuffer scale"),
        };

        let mut window = minifb::Window::new("RISCVBox", width, height, options)
            .expect("Failed to create window");

        window.set_target_fps(60);

        let fb_slice =
            unsafe { std::slice::from_raw_parts(fb_ptr as *const u32, width * height + 1) };

        window.set_input_callback(Box::new(UartCB {}));

        let mut this = Self {
            window,
            width,
            height,
            fb_slice,
            framebuffer: vec![0; width * height * 4],
        };

        this.set_icon();
        this.set_dark_mode();

        this
    }

    pub fn event_loop(&mut self) {
        while self.window.is_open() && !self.window.is_key_down(minifb::Key::Escape) {
            // Convert ABGR to BGR0

            for i in 0..self.width * self.height {
                let pixel = self.fb_slice[i];

                self.framebuffer[i] = ((pixel & 0x00ff_0000) >> 16)
                    | (pixel & 0x0000_ff00)
                    | ((pixel & 0x0000_00ff) << 16);
            }

            self.window
                .update_with_buffer(&self.framebuffer, self.width, self.height)
                .unwrap();
        }

        std::process::exit(0);
    }

    #[cfg(windows)]
    fn set_icon(&mut self) {
        use winapi::{
            shared::{ntdef::PCWSTR, windef::HWND},
            um::{
                libloaderapi::GetModuleHandleW,
                winuser::{
                    LoadImageW, SendMessageW, ICON_BIG, ICON_SMALL, IMAGE_ICON, LR_DEFAULTSIZE,
                    WM_SETICON,
                },
            },
        };

        let _icon = unsafe {
            let handle = GetModuleHandleW(std::ptr::null());

            if handle.is_null() {
                return;
            }

            let res = LoadImageW(handle, 1 as PCWSTR, IMAGE_ICON, 0, 0, LR_DEFAULTSIZE);

            if res.is_null() {
                return;
            }

            SendMessageW(
                self.window.get_window_handle() as HWND,
                WM_SETICON,
                ICON_SMALL as usize,
                res as isize,
            );

            SendMessageW(
                self.window.get_window_handle() as HWND,
                WM_SETICON,
                ICON_BIG as usize,
                res as isize,
            );
        };
    }

    #[cfg(windows)]
    fn set_dark_mode(&mut self) {
        use winapi::shared::minwindef::{BOOL, DWORD};
        use winapi::shared::windef::HWND;
        use winapi::um::dwmapi::DwmSetWindowAttribute;

        const DWMWA_USE_IMMERSIVE_DARK_MODE: DWORD = 20;
        const DWMWA_CAPTION_COLOR: DWORD = 35;

        let hwnd = self.window.get_window_handle();

        unsafe {
            let mut value = 1 as BOOL;

            DwmSetWindowAttribute(
                hwnd as HWND,
                DWMWA_USE_IMMERSIVE_DARK_MODE,
                &mut value as *mut _ as *mut _,
                std::mem::size_of_val(&value) as u32,
            );
        }
    }

    #[cfg(not(windows))]
    fn set_dark_mode(&mut self) {}

    #[cfg(not(windows))]
    fn set_icon(&mut self) {}
}
