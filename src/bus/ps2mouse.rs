use crate::window::{window_common::Ps2MouseEvent, window_impl, WindowCommon};
use crate::{bus::bus::*, cpu::Exception};

pub const PS2MOUSE_BEGIN_ADDR: BusType = 0x70;
pub const PS2MOUSE_END_ADDR: BusType = PS2MOUSE_BEGIN_ADDR + 0x10;

mod ps2_mouse_commands {
    pub const SET_SCALING_1_1: u32 = 0xE6;
    pub const SET_SCALING_2_1: u32 = 0xE7;
    pub const SET_RESOLUTION: u32 = 0xE8;
    pub const STATUS_REQ: u32 = 0xE9;
    pub const SET_STREAM_MODE: u32 = 0xEA;
    pub const READ_DATA: u32 = 0xEB;
    pub const RESET_WRAP_MODE: u32 = 0xEC;
    pub const SET_WRAP_MODE: u32 = 0xEE;
    pub const SET_REMOTE_MODE: u32 = 0xF0;
    pub const GET_DEV_ID: u32 = 0xF2;
    pub const SET_SAMPLE_RATE: u32 = 0xF3;
    pub const ENABLE_DATA_REPORTING: u32 = 0xF4;
    pub const DISABLE_DATA_REPORTING: u32 = 0xF5;
    pub const SET_DEFAULTS: u32 = 0xF6;
    pub const RESEND: u32 = 0xFE;
    pub const RESET: u32 = 0xFF;
}

mod ps2_mouse_reply {
    pub const ACK: u32 = 0xFA;
    pub const RESEND: u32 = 0xFE;
    pub const ERROR: u32 = 0xFC;
}

enum PS2MouseState {
    Command,
    SetSampleRate,
    SetWrap,
    SetResolution,
}

mod ps2_mouse_mode {
    pub const STREAM: u8 = 0;
    pub const REMOTE: u8 = 0x40;
}

mod ps2_mouse_buttons {
    pub const LEFT: u8 = 0x01;
    pub const RIGHT: u8 = 0x02;
    pub const MIDDLE: u8 = 0x04;
}

pub struct PS2Mouse {
    state: PS2MouseState,
    button_state: u8,
    mode: u8,
    resolution: u8,
    sample_rate: u8,
    wrap: bool,
    last_command: u32,
    is_reporting: bool,
    x: i32,
    y: i32,
    x_overflow: u8,
    y_overflow: u8,
    cmd_queue: std::collections::VecDeque<u8>,
}

impl PS2Mouse {
    pub fn new() -> Self {
        Self {
            state: PS2MouseState::Command,
            mode: ps2_mouse_mode::STREAM,
            button_state: 0,
            resolution: 2,
            sample_rate: 100,
            wrap: false,
            last_command: 0,
            is_reporting: true,
            x: 0,
            y: 0,
            x_overflow: 0,
            y_overflow: 0,
            cmd_queue: std::collections::VecDeque::new(),
        }
    }

    fn push_cmd(&mut self, cmd: u8) {
        self.cmd_queue.push_back(cmd);
    }

    fn pop_cmd(&mut self) -> Option<u8> {
        let cmd = self.cmd_queue.pop_front();

        if cmd.is_some() {
            self.last_command = cmd.unwrap() as u32;
        }

        cmd
    }

    fn send_mouse_packet(&mut self) {
        let x = self.x as u8;
        let y = self.y as u8;

        let x_sign = if self.x < 0 { 1 } else { 0 };
        let y_sign = if self.y < 0 { 1 } else { 0 };

        let cmd: u8 = self.button_state
            | 1 << 7
            | x_sign << 4
            | y_sign << 5
            | self.x_overflow << 6
            | self.y_overflow << 7;

        self.push_cmd(cmd);
        self.push_cmd(x);
        self.push_cmd(y);
    }

    fn handle_mouse_event(&mut self, event: Ps2MouseEvent) {
        match event {
            Ps2MouseEvent::Move { x, y } => {
                let resolution = 3 - self.resolution;

                self.x = x * resolution as i32;
                self.y = y * resolution as i32;

                if self.x > 0xFF {
                    self.x_overflow = 1;
                    self.x = 0xFF;
                } else if self.x < -0xFF {
                    self.x_overflow = 1;
                    self.x = -0xFF;
                } else {
                    self.x_overflow = 0;
                }

                if self.y > 0xFF {
                    self.y_overflow = 1;
                    self.y = 0xFF;
                } else if self.y < -0xFF {
                    self.y_overflow = 1;
                    self.y = -0xFF;
                } else {
                    self.y_overflow = 0;
                }

                self.send_mouse_packet();
            }
            Ps2MouseEvent::LeftDown => {
                self.button_state |= ps2_mouse_buttons::LEFT;
                self.send_mouse_packet();
            }
            Ps2MouseEvent::LeftUp => {
                self.button_state &= !ps2_mouse_buttons::LEFT;
                self.send_mouse_packet();
            }
            Ps2MouseEvent::RightDown => {
                self.button_state |= ps2_mouse_buttons::RIGHT;
                self.send_mouse_packet();
            }
            Ps2MouseEvent::RightUp => {
                self.button_state &= !ps2_mouse_buttons::RIGHT;
                self.send_mouse_packet();
            }
            Ps2MouseEvent::MiddleDown => {
                self.button_state |= ps2_mouse_buttons::MIDDLE;
                self.send_mouse_packet();
            }
            Ps2MouseEvent::MiddleUp => {
                self.button_state &= !ps2_mouse_buttons::MIDDLE;
                self.send_mouse_packet();
            }
            Ps2MouseEvent::WheelUp { delta: _ } => {}
            Ps2MouseEvent::WheelDown { delta: _ } => {}
        }
    }
}

impl BusDevice for PS2Mouse {
    fn load(&mut self, _addr: BusType, _size: BusType) -> Result<BusType, Exception> {
        if self.is_reporting {
            loop {
                let event = window_impl::get_mouse_event();
                if event.is_none() {
                    break;
                }
                let event = event.unwrap();
                self.handle_mouse_event(event);
            }
        }

        if self.cmd_queue.len() > 0 {
            return Ok(self.pop_cmd().unwrap() as BusType);
        }

        Ok(0)
    }

    fn store(&mut self, _addr: BusType, _data: BusType, _size: BusType) -> Result<(), Exception> {
        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        return PS2MOUSE_BEGIN_ADDR;
    }

    fn get_end_addr(&self) -> BusType {
        return PS2MOUSE_END_ADDR;
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}
}
