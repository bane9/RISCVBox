use crate::backend::{
    common::{BackendCore, PtrT},
    HostEncodedInsn,
};
use crate::cpu::*;

const MAX_WALK_BACK: usize = 100;

// Callee needs to `use std::arch::asm;`
#[macro_export]
macro_rules! host_get_return_addr {
    () => {{
        let ret: *mut u8;

        unsafe {
            asm!(
                "mov {0}, [rbp - 8]",
                out(reg) ret,
                options(nostack, preserves_flags)
            );
        }

        ret
    }};
}

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

    #[rustfmt::skip]
    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn {
        
        let mut fn_call = [
            0x55, // push rbp
            
            0x48, 0x89, 0xE5, // mov rbp, rsp
            
            0x49, 0xBB, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, // mov r11, 0x00
            
            0x41, 0xFF, 0xD3, // call r11
            
            0x5D // pop rbp
        ];
        
        let fn_addr_bytes: [u8; 8] = unsafe { std::mem::transmute(fn_ptr as usize) };
        
        fn_call[6..14].copy_from_slice(fn_addr_bytes.as_ref());
    
        HostEncodedInsn::new_from_slice(&fn_call)
    }
    
    fn find_guest_pc_from_host_stack_frame(caller_ret_addr: *mut u8) -> Option<u32> {
        let cpu = cpu::get_cpu();

        for i in 0..MAX_WALK_BACK {
            let addr = caller_ret_addr.wrapping_sub(i);

            if let Some(guest_pc) = cpu.insn_map.get_by_key(addr) {
                return Some(*guest_pc);
            }
        }

        None
    }
}
