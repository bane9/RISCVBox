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
}
