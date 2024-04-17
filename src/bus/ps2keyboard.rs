use crate::cpu;
use crate::window::window_common::Ps2Key;
use crate::{bus::bus::*, cpu::Exception};

pub const PS2KEYBOARD_BEGIN_ADDR: BusType = 0x60;
pub const PS2KEYBOARD_END_ADDR: BusType = PS2KEYBOARD_BEGIN_ADDR + 0x10;

use crate::window::{window_impl, WindowCommon};

mod ps2cmd {
    pub const SETLED: u8 = 0xed;
    pub const ECHO: u8 = 0xee;
    pub const SCANCODESET: u8 = 0xf0;
    pub const ID: u8 = 0xf2;
    pub const TYPEMATIC: u8 = 0xf3;
    pub const ENABLE: u8 = 0xf4;
    pub const DISABLE: u8 = 0xf5;
    pub const DEFAULT: u8 = 0xf6;
    pub const ALLTYPEMATIC: u8 = 0xf7;
    pub const ALLMAKEBREAK: u8 = 0xf8;
    pub const ALLMAKE: u8 = 0xf9;
    pub const ALLMAKEBREAKAUTOREPEAT: u8 = 0xfa;
    pub const RESEND: u8 = 0xfe;
    pub const RESET: u8 = 0xff;
}

mod ps2_reply {
    pub const ACK: u8 = 0xfa;
    pub const RESEND: u8 = 0xfe;
    pub const ERROR: u8 = 0xfc;
}

mod ps2_state {
    pub const CMD: u8 = 0;
    pub const SET_SAMPLE_RATE: u8 = 1;
    pub const SET_SCAN_CODE_SET: u8 = 2;
    pub const SET_LED: u8 = 3;
}

pub struct PS2Keyboard {
    state: u8,
    rate: u8,
    delay: u8,
    reporting: bool,

    last_key: Ps2Key,

    cmd_queue: std::collections::VecDeque<u8>,
    last_command: u8,
}

impl PS2Keyboard {
    pub fn new() -> Self {
        Self {
            state: ps2_state::CMD,
            rate: 20,
            delay: 1,
            reporting: false,
            last_key: Ps2Key::default(),

            cmd_queue: std::collections::VecDeque::new(),
            last_command: 0,
        }
    }

    fn push_cmd(&mut self, cmd: u8) {
        self.cmd_queue.push_back(cmd);
    }

    fn pop_cmd(&mut self) -> Option<u8> {
        let cmd = self.cmd_queue.pop_front();

        if cmd.is_some() {
            self.last_command = cmd.unwrap();
        }

        cmd
    }

    fn handle_cmd(&mut self, cmd: u8) {
        match cmd {
            ps2cmd::DEFAULT => {
                self.rate = 20;
                self.delay = 1;

                self.push_cmd(ps2_reply::ACK);
            }
            ps2cmd::DISABLE => {
                self.reporting = false;
                self.push_cmd(ps2_reply::ACK);
            }
            ps2cmd::ENABLE => {
                self.reporting = true;
                self.push_cmd(ps2_reply::ACK);
            }
            ps2cmd::ECHO => {
                self.push_cmd(ps2cmd::ECHO);
            }
            ps2cmd::ID => {
                self.push_cmd(ps2_reply::ACK);
                self.push_cmd(0xab);
                self.push_cmd(0x83);
            }
            ps2cmd::RESEND => {
                self.push_cmd(self.last_command);
            }
            ps2cmd::SETLED => {
                self.state = ps2_state::SET_LED;
                self.push_cmd(ps2_reply::ACK);
            }
            ps2cmd::SCANCODESET => {
                self.state = ps2_state::SET_SCAN_CODE_SET;
                self.push_cmd(ps2_reply::ACK);
            }
            _ => {
                self.cmd_queue.push_back(ps2_reply::RESEND);
            }
        }
    }
}

impl BusDevice for PS2Keyboard {
    fn load(&mut self, _addr: BusType, _size: BusType) -> Result<BusType, Exception> {
        let key = window_impl::get_key();

        if let Some(key) = key {
            for x in key.iter() {
                if *x != 0 {
                    self.push_cmd(*x);
                }
            }

            self.last_key = key;
        }

        if self.cmd_queue.len() > 0 {
            return Ok(self.pop_cmd().unwrap() as BusType);
        }

        Ok(0)
    }

    fn store(&mut self, _addr: BusType, data: BusType, _size: BusType) -> Result<(), Exception> {
        match self.state {
            ps2_state::CMD => {
                self.handle_cmd(data as u8);
            }
            ps2_state::SET_LED => {
                self.state = ps2_state::CMD;
                self.push_cmd(ps2_reply::ACK);
            }
            ps2_state::SET_SCAN_CODE_SET => {
                if data == 0 {
                    self.state = ps2_state::CMD;
                    self.push_cmd(ps2_reply::ACK);
                    self.push_cmd(2);
                } else if data == 2 {
                    self.state = ps2_state::SET_SAMPLE_RATE;
                    self.push_cmd(ps2_reply::ACK);
                } else {
                    self.state = ps2_state::CMD;
                    self.push_cmd(ps2_reply::RESEND);
                }

                self.state = ps2_state::CMD;
                self.push_cmd(ps2_reply::ACK);
            }
            ps2_state::SET_SAMPLE_RATE => {
                self.state = ps2_state::CMD;
                self.rate = (data & 0x1f) as u8;
                self.delay = (data & 3) as u8;
                self.push_cmd(ps2_reply::ACK);
            }
            _ => {
                self.state = ps2_state::CMD;
                self.push_cmd(ps2_reply::ACK);
            }
        }

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        PS2KEYBOARD_BEGIN_ADDR
    }

    fn get_end_addr(&self) -> BusType {
        PS2KEYBOARD_END_ADDR
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}

    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        None
    }
}
