use crate::backend::{
    common::{BackendCore, PtrT},
    DecodeRet, HostEncodedInsn, HostInsnT,
};

pub struct BackendCoreImpl;

impl BackendCore for BackendCoreImpl {
    fn fill_with_target_nop(ptr: PtrT, size: usize) {
        static NOP: [u8; 1] = [0x90];

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

    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn {
        let mut fn_call = [
            0x49 as u8, 0xBB, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x42, 0x41, 0xFF, 0xD3,
        ];
        let fn_as_u8 = fn_ptr as *mut u8;

        for i in 0..8 {
            fn_call[2 + i] = unsafe { *fn_as_u8.add(i) };
        }

        HostEncodedInsn::new_from_slice(fn_call.as_ref())
    }
}
