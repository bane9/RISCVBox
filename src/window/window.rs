use minifb;

pub struct Window {
    window: minifb::Window,
    width: usize,
    height: usize,
    fb_slice: &'static [u32],
    framebuffer: Vec<u32>,
}

impl Window {
    pub fn new(fb_ptr: *mut u8, width: usize, height: usize) -> Self {
        let options = minifb::WindowOptions::default();

        let mut window = minifb::Window::new("RISCVBox", width, height, options)
            .expect("Failed to create window");

        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));

        let fb_slice =
            unsafe { std::slice::from_raw_parts(fb_ptr as *const u32, width * height + 1) };

        Self {
            window,
            width,
            height,
            fb_slice,
            framebuffer: vec![0; width * height * 4],
        }
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
    }
}
