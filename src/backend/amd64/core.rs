use crate::backend::{
    common::{BackendCore, PtrT},
    HostEncodedInsn,
};
use crate::cpu::*;

use std::arch::asm;

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

pub mod amd64_reg {
    pub const RAX: u8 = 0;
    pub const RCX: u8 = 1;
    pub const RDX: u8 = 2;
    pub const RBX: u8 = 3;
    pub const RSP: u8 = 4;
    pub const RBP: u8 = 5;
    pub const RSI: u8 = 6;
    pub const RDI: u8 = 7;
    pub const R8: u8 = 8;
    pub const R9: u8 = 9;
    pub const R10: u8 = 10;
    pub const R11: u8 = 11;
    pub const R12: u8 = 12;
    pub const R13: u8 = 13;
    pub const R14: u8 = 14;
    pub const R15: u8 = 15;
}

#[macro_export]
macro_rules! emit_insn {
    ($enc:expr, $insn:expr) => {{
        $enc.push_slice($insn.as_ref());
    }};
}

#[macro_export]
macro_rules! emit_push_reg {
    ($enc:expr, $reg:expr) => {{
        emit_insn!($enc, [0x50 + $reg as u8]);
    }};
}

#[macro_export]
macro_rules! emit_mov_reg_reg {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        emit_insn!($enc, [0x48, 0x89, 0xC0 + ($src_reg << 3) + $dst_reg]);
    }};
}

#[macro_export]
macro_rules! emit_pop_reg {
    ($enc:expr, $reg:expr) => {{
        emit_insn!($enc, [0x58 + $reg as u8]);
    }};
}

#[macro_export]
macro_rules! emit_move_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        emit_insn!($enc, [0x49, 0xB0 + $reg as u8]);
        emit_insn!($enc, (($imm) as usize).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_call_reg {
    ($enc:expr, $reg:expr) => {{
        emit_insn!($enc, [0x41, 0xFF, 0xc8 + $reg as u8]);
    }};
}

#[macro_export]
macro_rules! emit_nop {
    ($enc:expr) => {{
        emit_insn!($enc, [0x90]);
    }};
}

#[macro_export]
macro_rules! emit_ret {
    ($enc:expr) => {{
        emit_insn!($enc, [0xc3]);
    }};
}

#[macro_export]
macro_rules! emit_mov_qword_ptr {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        emit_insn!(
            $enc,
            [
                0x48 + ($reg > amd64_reg::R8) as u8,
                0xC7,
                ($reg as u8) - (($reg > amd64_reg::R8) as u8 * amd64_reg::R8)
            ]
        );
        emit_insn!($enc, (($imm) as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_dword_ptr {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if reg < amd64_reg::R8 {
            emit_insn!($enc, [0xC7, $reg as u8]);
        } else {
            emit_insn!($enc, [0x41, 0xC7, $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, (($imm) as u32).to_le_bytes());
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

    fn fill_with_target_ret(ptr: PtrT, size: usize) {
        static RET: [u8; 1] = [0xc3];

        for i in 0..(size / RET.len()) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    RET.as_ptr(),
                    (ptr.wrapping_add(i * RET.len())) as *mut u8,
                    RET.len(),
                );
            }
        }
    }

    fn emit_ret_with_status(state: RunState) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        let cpu = cpu::get_cpu();

        emit_move_reg_imm!(insn, amd64_reg::R11, &cpu.ret_status as *const _);
        emit_mov_qword_ptr!(insn, amd64_reg::R11, state as u32);
        emit_ret!(insn);

        insn
    }

    #[rustfmt::skip]
    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_push_reg!(insn, amd64_reg::RBP);
        emit_mov_reg_reg!(insn, amd64_reg::RBP, amd64_reg::RSP);
        emit_move_reg_imm!(insn, amd64_reg::R11, fn_ptr);
        emit_call_reg!(insn, amd64_reg::R11);
        emit_pop_reg!(insn, amd64_reg::RBP);

        insn
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
