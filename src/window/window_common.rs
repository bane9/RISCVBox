#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ps2MouseEvent {
    Move { x: i32, y: i32 },
    LeftDown,
    LeftUp,
    RightDown,
    RightUp,
    MiddleDown,
    MiddleUp,
    WheelUp { delta: i32 },
    WheelDown { delta: i32 },
}

pub type Ps2Key = [u8; 3];

pub trait WindowCommon {
    fn new(
        width: usize,
        height: usize,
        bpp: usize,
        title: &str,
        fb_ptr: *mut u8,
        hide_window: bool,
    ) -> Self;
    fn event_loop(&mut self);
    fn get_key() -> Option<Ps2Key>;
    fn get_mouse_event() -> Option<Ps2MouseEvent>;
}
