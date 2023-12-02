pub type Ps2Key = u16;
pub type Ps2Mouse = u16;

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
    fn get_mouse() -> Option<Ps2Mouse>;
}
