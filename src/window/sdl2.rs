use crate::window::window_common::*;

use std::time::{Duration, Instant};

use lazy_static::lazy_static;
use multiqueue::{broadcast_queue, BroadcastReceiver, BroadcastSender};
use sdl2::keyboard::Keycode;
use std::sync::Mutex;

use sdl2::event::*;
use sdl2::render::*;
use sdl2::video::*;

const PS2_QUEUE_SIZE: u64 = 1024;

const LOGIC_UPDATE_HZ: u32 = 60;

lazy_static! {
    static ref KEYBOARD: Mutex<(BroadcastSender<Ps2Key>, BroadcastReceiver<Ps2Key>)> = {
        let (sender, receiver) = broadcast_queue::<Ps2Key>(PS2_QUEUE_SIZE);
        Mutex::new((sender, receiver))
    };
    static ref MOUSE: Mutex<(BroadcastSender<Ps2Key>, BroadcastReceiver<Ps2Key>)> = {
        let (sender, receiver) = broadcast_queue::<Ps2Key>(PS2_QUEUE_SIZE);
        Mutex::new((sender, receiver))
    };
}

pub struct Sdl2Window {
    pub fb_ptr: *mut u8,
    pub width: usize,
    pub height: usize,
    pub bpp: usize,
    pub is_hidden: bool,

    sdl2_context: sdl2::Sdl,
    video_subsystem: sdl2::VideoSubsystem,
    canvas: Canvas<Window>,
    texture_creator: TextureCreator<WindowContext>,
}

impl Sdl2Window {
    pub fn update(&mut self) {
        let bpp = self.bpp / 8;

        let pixel_fomat = match bpp {
            2 => sdl2::pixels::PixelFormatEnum::RGB565,
            4 => sdl2::pixels::PixelFormatEnum::RGBA8888,
            _ => panic!("Unsupported bpp"),
        };

        let mut texture = self
            .texture_creator
            .create_texture_streaming(pixel_fomat, self.width as u32, self.height as u32)
            .unwrap();

        unsafe {
            let data = std::slice::from_raw_parts(self.fb_ptr, self.width * self.height * bpp);

            texture.update(None, data, self.width * bpp).unwrap();
        }

        self.canvas.copy(&texture, None, None).unwrap();
        self.canvas.present();
    }

    fn sdl2key_to_ps2key(key: Keycode) -> Option<Ps2Key> {
        match key {
            Keycode::A => Some(0x1c),
            Keycode::B => Some(0x32),
            Keycode::C => Some(0x21),
            Keycode::D => Some(0x23),
            Keycode::E => Some(0x24),
            Keycode::F => Some(0x2b),
            Keycode::G => Some(0x34),
            Keycode::H => Some(0x33),
            Keycode::I => Some(0x43),
            Keycode::J => Some(0x3b),
            Keycode::K => Some(0x42),
            Keycode::L => Some(0x4b),
            Keycode::M => Some(0x3a),
            Keycode::N => Some(0x31),
            Keycode::O => Some(0x44),
            Keycode::P => Some(0x4d),
            Keycode::Q => Some(0x15),
            Keycode::R => Some(0x2d),
            Keycode::S => Some(0x1b),
            Keycode::T => Some(0x2c),
            Keycode::U => Some(0x3c),
            Keycode::V => Some(0x2a),
            Keycode::W => Some(0x1d),
            Keycode::X => Some(0x22),
            Keycode::Y => Some(0x35),
            Keycode::Z => Some(0x1a),
            Keycode::Num0 => Some(0x45),
            Keycode::Num1 => Some(0x16),
            Keycode::Num2 => Some(0x1e),
            Keycode::Num3 => Some(0x26),
            Keycode::Num4 => Some(0x25),
            Keycode::Num5 => Some(0x2e),
            Keycode::Num6 => Some(0x36),
            Keycode::Num7 => Some(0x3d),
            Keycode::Num8 => Some(0x3e),
            Keycode::Num9 => Some(0x46),
            Keycode::Escape => Some(0x76),
            Keycode::LShift => Some(0x12),
            Keycode::LAlt => Some(0x11),
            Keycode::LGui => Some(0x1f),
            Keycode::RShift => Some(0x59),
            Keycode::RAlt => Some(0x11),
            Keycode::RGui => Some(0x27),
            Keycode::Return => Some(0x5a),
            Keycode::Space => Some(0x29),
            Keycode::Backspace => Some(0x66),
            Keycode::Tab => Some(0x0d),
            Keycode::Minus => Some(0x4e),
            Keycode::Equals => Some(0x55),
            Keycode::LeftBracket => Some(0x54),
            Keycode::RightBracket => Some(0x5b),
            Keycode::Backslash => Some(0x5d),
            Keycode::Semicolon => Some(0x4c),
            Keycode::Comma => Some(0x41),
            Keycode::Period => Some(0x49),
            Keycode::Slash => Some(0x4a),
            Keycode::CapsLock => Some(0x58),
            Keycode::F1 => Some(0x05),
            Keycode::F2 => Some(0x06),
            Keycode::F3 => Some(0x04),
            Keycode::F4 => Some(0x0c),
            Keycode::F5 => Some(0x03),
            Keycode::F6 => Some(0x0b),
            Keycode::F7 => Some(0x83),
            Keycode::F8 => Some(0x0a),
            Keycode::F9 => Some(0x01),
            Keycode::F10 => Some(0x09),
            Keycode::F11 => Some(0x78),
            Keycode::F12 => Some(0x07),
            Keycode::PrintScreen => Some(0xe012),
            Keycode::ScrollLock => Some(0x7e),
            Keycode::Pause => Some(0xe114),
            Keycode::Insert => Some(0xe070),
            Keycode::Home => Some(0xe06c),
            Keycode::PageUp => Some(0xe07d),
            Keycode::Delete => Some(0xe071),
            Keycode::End => Some(0xe069),
            Keycode::PageDown => Some(0xe07a),
            Keycode::Right => Some(0xe074),
            Keycode::Left => Some(0xe06b),
            Keycode::Down => Some(0xe072),
            Keycode::Up => Some(0xe075),
            Keycode::NumLockClear => Some(0x77),
            Keycode::KpDivide => Some(0xe04a),
            Keycode::KpMultiply => Some(0x7c),
            Keycode::KpMinus => Some(0x7b),
            Keycode::KpPlus => Some(0x79),
            Keycode::KpEnter => Some(0xe05a),
            Keycode::Kp1 => Some(0x69),
            Keycode::Kp2 => Some(0x72),
            Keycode::Kp3 => Some(0x7a),
            Keycode::Kp4 => Some(0x6b),
            Keycode::Kp5 => Some(0x73),
            Keycode::Kp6 => Some(0x74),
            Keycode::Kp7 => Some(0x6c),
            Keycode::Kp8 => Some(0x75),
            Keycode::Kp9 => Some(0x7d),
            Keycode::Kp0 => Some(0x70),
            Keycode::KpPeriod => Some(0x71),
            Keycode::Application => Some(0x2f),
            Keycode::Power => Some(0xe037),
            Keycode::KpEquals => Some(0x67),
            _ => None,
        }
    }
}

impl WindowCommon for Sdl2Window {
    fn new(
        width: usize,
        height: usize,
        bpp: usize,
        title: &str,
        fb_ptr: *mut u8,
        hide_window: bool,
    ) -> Self {
        let sdl2_context = sdl2::init().unwrap();
        let video_subsystem = sdl2_context.video().unwrap();

        let mut binding = video_subsystem.window(title, width as u32, height as u32);
        let window = binding.position_centered();

        let window = if hide_window { window.hidden() } else { window };

        let window = window.build().unwrap();

        let canvas = window.into_canvas().build().unwrap();
        let texture_creator = canvas.texture_creator();

        Sdl2Window {
            fb_ptr,
            width,
            height,
            bpp,
            is_hidden: hide_window,

            sdl2_context,
            video_subsystem,
            canvas,
            texture_creator,
        }
    }

    fn event_loop(&mut self) {
        if self.is_hidden {
            return;
        }

        let mut event_pump = self.sdl2_context.event_pump().unwrap();
        let mut last_update = Instant::now();
        let frame_duration = Duration::new(0, 1_000_000_000u32 / LOGIC_UPDATE_HZ);

        loop {
            for event in event_pump.poll_iter() {
                match event {
                    Event::KeyDown {
                        timestamp: _,
                        window_id: _,
                        keycode,
                        scancode: _,
                        keymod: _,
                        repeat: _,
                    } => {
                        let ps2key = Sdl2Window::sdl2key_to_ps2key(keycode.unwrap());

                        if let Some(ps2key) = ps2key {
                            KEYBOARD.lock().unwrap().0.try_send(ps2key).unwrap();
                        }
                    }
                    Event::Quit { .. } => std::process::exit(0),
                    _ => {}
                }
            }

            self.update();

            let elapsed = last_update.elapsed();
            if let Some(sleep_duration) = frame_duration.checked_sub(elapsed) {
                std::thread::sleep(sleep_duration);
            }

            last_update = Instant::now();
        }
    }

    fn get_key() -> Option<Ps2Key> {
        let receiver = KEYBOARD.lock().unwrap().1.add_stream();

        match receiver.try_recv() {
            Ok(key) => Some(key),
            Err(_) => None,
        }
    }

    fn get_mouse() -> Option<Ps2Mouse> {
        let receiver = MOUSE.lock().unwrap().1.add_stream();

        match receiver.try_recv() {
            Ok(key) => Some(key),
            Err(_) => None,
        }
    }
}
