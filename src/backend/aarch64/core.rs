use crate::backend::common::{BackendCore, PtrT};

pub struct BackendCoreImpl;

impl BackendCore for BackendCoreImpl {
    fn fill_with_target_nop(ptr: PtrT, size: usize) {
        static NOP: [u8; 4] = [0x1f, 0x20, 0x03, 0xd5];
        assert!(size % NOP.len() == 0);

        for i in 0..(size / NOP.len()) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    NOP.as_ptr(),
                    (ptr.wrapping_add(i * NOP.len())) as *mut u8,
                    NOP.len(),
                );
            }
        }
    }

    fn fill_with_target_exc(ptr: PtrT, size: usize) {
        static BRK0: [u8; 4] = [0x00, 0x00, 0x20, 0xd4];
        assert!(size % BRK0.len() == 0);

        for i in 0..(size / BRK0.len()) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    BRK0.as_ptr(),
                    (ptr.wrapping_add(i * BRK0.len())) as *mut u8,
                    BRK0.len(),
                );
            }
        }
    }
}
