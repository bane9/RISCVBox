#[cfg(windows)]
pub mod sdl2_dark_mode_win {
    extern crate sdl2;

    use sdl2::sys::SDL_GetWindowWMInfo;
    use sdl2::sys::SDL_SysWMinfo;
    use sdl2::video::Window;
    use winapi::shared::minwindef::{DWORD, HINSTANCE};
    use winapi::shared::windef::{HDC, HWND};
    use winapi::{
        shared::winerror::{HRESULT, S_OK},
        um::dwmapi::DwmSetWindowAttribute,
    };

    // Both of these require Windows 11
    // https://learn.microsoft.com/en-us/windows/win32/api/dwmapi/ne-dwmapi-dwmwindowattribute
    pub const DWMWA_USE_IMMERSIVE_DARK_MODE: DWORD = 20;
    pub const DWMWA_CAPTION_COLOR: DWORD = 35;

    #[repr(C)]
    struct Win {
        hwnd: HWND,
        hdc: HDC,
        hinstance: HINSTANCE,
    }

    fn get_hwnd(window: &Window) -> HWND {
        use winapi::um::winuser::IsWindow;

        unsafe {
            let mut wm_info = std::mem::zeroed::<SDL_SysWMinfo>();
            if SDL_GetWindowWMInfo(window.raw(), &mut wm_info) == sdl2::sys::SDL_bool::SDL_TRUE {
                let win = std::mem::transmute::<_, *mut Win>(&wm_info.info as *const _);

                if IsWindow((*win).hwnd) == 0 {
                    return 0 as HWND;
                }

                (*win).hwnd
            } else {
                0 as HWND
            }
        }
    }

    fn enable_dark_mode(hwnd: HWND) -> Result<(), HRESULT> {
        unsafe {
            let mut value = 0x2E2E2E as DWORD;

            let ret = DwmSetWindowAttribute(
                hwnd,
                DWMWA_CAPTION_COLOR,
                &mut value as *mut _ as *mut _,
                std::mem::size_of_val(&value) as u32,
            );

            if ret == S_OK {
                Ok(())
            } else {
                Err(ret)
            }
        }
    }

    pub fn enable_dark_mode_for_window(window: &Window) {
        let hwnd = get_hwnd(window);

        if hwnd == 0 as HWND {
            return;
        }

        let _ = enable_dark_mode(hwnd);
    }
}

#[cfg(not(windows))]
pub mod sdl2_dark_mode_nop {
    use sdl2::video::Window;

    pub fn enable_dark_mode_for_window_nop(_window: &Window) {}
}
