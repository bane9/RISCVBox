use crate::backend::{
    common::{BackendCore, PtrT},
    HostEncodedInsn,
};
use crate::cpu::*;

use std::arch::asm;

pub type PtrT = *mut u8;
pub type HostInsnT = u8;
pub const HOST_INSN_MAX_SIZE: usize = 96;
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;

// Callee needs to `use std::arch::asm;`
#[macro_export]
macro_rules! host_get_return_addr {
    () => {{
        let ret: *mut u8;

        unsafe {
            asm!(
                "mov {0}, fp",
                out(reg) ret,
                options(nostack, preserves_flags)
            );
        }

        ret
    }};
}

pub struct BackendCoreImpl;

pub mod aarch64_reg {
    pub const X0: u32 = 0;
    pub const X1: u32 = 1;
    pub const X2: u32 = 2;
    pub const X3: u32 = 3;
    pub const X4: u32 = 4;
    pub const X5: u32 = 5;
    pub const X6: u32 = 6;
    pub const X7: u32 = 7;
    pub const X8: u32 = 8;
    pub const X9: u32 = 9;
    pub const X10: u32 = 10;
    pub const X11: u32 = 11;
    pub const X12: u32 = 12;
    pub const X13: u32 = 13;
    pub const X14: u32 = 14;
    pub const X15: u32 = 15;
    pub const X16: u32 = 16;
    pub const X17: u32 = 17;
    pub const X18: u32 = 18;
    pub const X19: u32 = 19;
    pub const X20: u32 = 20;
    pub const X21: u32 = 21;
    pub const X22: u32 = 22;
    pub const X23: u32 = 23;
    pub const X24: u32 = 24;
    pub const X25: u32 = 25;
    pub const X26: u32 = 26;
    pub const X27: u32 = 27;
    pub const X28: u32 = 28;
    pub const X29: u32 = 29;
    pub const X30: u32 = 30;
    pub const X31: u32 = 31;
    pub const FP: u32 = X29;
    pub const RA: u32 = X30;
    pub const SP: u32 = X31;
    pub const XZR: u32 = X31;
}

#[cfg(unix)]
pub mod aarch64_abi {
    pub use super::aarch64_reg;

    pub const ARG0: u32 = aarch64_reg::X0;
    pub const ARG1: u32 = aarch64_reg::X1;
    pub const ARG2: u32 = aarch64_reg::X2;
    pub const ARG3: u32 = aarch64_reg::X3;
    pub const ARG4: u32 = aarch64_reg::X4;
    pub const ARG5: u32 = aarch64_reg::X5;
    pub const ARG6: u32 = aarch64_reg::X6;
    pub const ARG7: u32 = aarch64_reg::X7;
    pub const ARG8: u32 = aarch64_reg::X8;
    pub const RET0: u32 = aarch64_reg::X0;
    pub const RET1: u32 = aarch64_reg::X1;
}

#[cfg(windows)]
pub mod aarch64_abi {
    pub use super::aarch64_reg;

    pub const ARG0: u32 = aarch64_reg::X0;
    pub const ARG1: u32 = aarch64_reg::X1;
    pub const ARG2: u32 = aarch64_reg::X2;
    pub const ARG3: u32 = aarch64_reg::X3;
    pub const RET0: u32 = aarch64_reg::X0;
    pub const RET1: u32 = aarch64_reg::X1;
}

pub mod store_size {
    pub const BYTE: u32 = 0;
    pub const HALFWORD: u32 = 1;
    pub const WORD: u32 = 2;
    pub const DOUBLEWORD: u32 = 3;
}

#[macro_export]
macro_rules! emit_insn {
    ($enc:expr, $insn:expr) => {{
        $enc.push_slice($insn.as_ref());
    }};
}

#[macro_export]
macro_rules! emit_add_reg_imm {
    ($enc:expr, $rd:expr, $rn:expr, $imm:expr) => {{
        let encoding: u32 =
            (1 << 31)        // sf=1 for 64-bit
            | (0 << 30)      // op=1 for SUB
            | (0 << 29)      // S=0 for not setting flags
            | (0b10001 << 24)// 10001 for SUB
            | (0 << 22)      // shift=00 (LSL #0)
            | (($imm & 0xFFF) << 10) // 12-bit immediate
            | (($rn as u32) << 5)
            | ($rd as u32);

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_sub_reg_imm {
    ($enc:expr, $rd:expr, $rn:expr, $imm:expr) => {{
        let encoding: u32 =
            (1 << 31)        // sf=1 for 64-bit
            | (1 << 30)      // op=1 for SUB
            | (0 << 29)      // S=0 for not setting flags
            | (0b10001 << 24)// 10001 for SUB
            | (0 << 22)      // shift=00 (LSL #0)
            | (($imm & 0xFFF) << 10) // 12-bit immediate
            | (($rn as u32) << 5)
            | ($rd as u32);

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_str_reg {
    ($enc:expr, $size:expr, $rn:expr, $rt:expr) => {{
        let encoding: u32 =
            ($size << 30) | (0b111000000) << 21 | (($rn as u32) << 5) | ($rt as u32);

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_str_reg_imm {
    ($enc:expr, $rt:expr, $rn:expr, $imm:expr) => {{
        let imm_copy = $imm;
        let imm_copy = imm_copy as u32;

        let encoding: u32 = (0b11 << 30)
            | (0b111000000 << 21)
            | ((imm_copy & 0x1ff) << 12)
            | (0b10 << 10)
            | (($rn as u32) << 5)
            | ($rt as u32);

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_ldr_reg_imm {
    ($enc:expr, $rt:expr, $rn:expr, $imm:expr) => {{
        let imm_copy = $imm;
        let imm_copy = imm_copy as u32;

        let encoding: u32 = (0b11 << 30)
            | (0b111000010 << 21)
            | ((imm_copy & 0x1ff) << 12)
            | (0b10 << 10)
            | (($rn as u32) << 5)
            | ($rt as u32);

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_reg_reg {
    ($enc:expr, $rd:expr, $rm:expr) => {{
        let sf = 1 << 31;
        let opc = 0b0101010 << 24;
        let shift = 0b00 << 22;
        let n = 0 << 21;
        let rm = ($rm & 0x1F) << 16;
        let imm = 0 << 13;
        let rn = 0b11111 << 5;
        let rd = $rd & 0x1F;

        let encoding: u32 = sf | opc | shift | n | rm | imm | rn | rd;

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_reg_sp {
    ($enc:expr, $rd:expr, $rn:expr) => {{
        let sf = 1 << 31;
        let op = 0 << 30;
        let s = 0b100010 << 23;
        let sh = 0 << 22;
        let imm12 = 0 << 10;
        let rn = $rn << 5;
        let rd = $rd & 0x1F;

        let encoding: u32 = sf | op | s | sh | imm12 | rn | rd;

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_movz {
    ($enc:expr, $dst_reg:expr, $imm:expr, $shift:expr) => {{
        let dst_reg = ($dst_reg & 0x1F) as u32;
        let imm = ($imm & 0xFFFF) as u32;
        let shift = (($shift / 16) & 0x3) as u32;

        let encoding: u32 = (0b110100101 << 23) | (shift << 21) | (imm << 5) | dst_reg;

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_movn {
    ($enc:expr, $dst_reg:expr, $imm:expr, $shift:expr) => {{
        let dst_reg = ($dst_reg & 0x1F) as u32;
        let imm = ($imm & 0xFFFF) as u32;
        let shift = (($shift / 16) & 0x3) as u32;

        let encoding: u32 = (0b110100101 << 23) | (shift << 21) | (imm << 5) | dst_reg;

        let encoding = encoding ^ (1 << 30);

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_movk {
    ($enc:expr, $dst_reg:expr, $imm:expr, $shift:expr) => {{
        let dst_reg = ($dst_reg & 0x1F) as u32;
        let imm = ($imm & 0xFFFF) as u32;
        let shift = (($shift / 16) & 0x3) as u32;

        let encoding: u32 = (0b111100101 << 23) | (shift << 21) | (imm << 5) | dst_reg;

        emit_insn!($enc, encoding.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_move_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        let imm: u64 = $imm as u64; // Avoid compiler warnings

        emit_movz!($enc, $reg, $imm, 0);
        if imm > 0xFFFF {
            emit_movk!($enc, $reg, imm >> 16, 16);
        }
        if imm > 0xFFFFFFFF {
            emit_movk!($enc, $reg, imm >> 32, 32);
        }
        if imm > 0xFFFFFFFFFFFF {
            emit_movk!($enc, $reg, imm >> 48, 48);
        }
    }};
}

#[macro_export]
macro_rules! emit_call_reg {
    ($enc:expr, $reg:expr) => {{
        let encoded = (0b1101011 << 25)
            | (0 << 24)
            | (0b00 << 22)
            | (1 << 21)
            | (0b11111 << 16)
            | (0b000000 << 10)
            | (($reg as u32 & 0b11111) << 5);
        emit_insn!($enc, encoded.to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_nop {
    ($enc:expr) => {{
        emit_insn!($enc, [0x1f, 0x20, 0x03, 0xd5]);
    }};
}

#[macro_export]
macro_rules! emit_ret {
    ($enc:expr) => {{
        emit_insn!($enc, [0xC0, 0x03, 0x5F, 0xD6]);
    }};
}

impl BackendCore for BackendCoreImpl {
    fn fill_with_target_nop(ptr: PtrT, size: usize) {
        static NOP: [u8; 4] = [0x1f, 0x20, 0x03, 0xd5];

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
        static RET: [u8; 4] = [0xc0, 0x03, 0x5f, 0xd6];

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

    fn emit_ret_with_status(state: crate::cpu::RunState) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_move_reg_imm!(insn, aarch64_reg::X9, state as u32);
        emit_move_reg_imm!(
            insn,
            aarch64_reg::X10,
            &cpu::get_cpu().ret_status as *const _ as usize
        );
        emit_str_reg!(
            insn,
            store_size::DOUBLEWORD,
            aarch64_reg::X10,
            aarch64_reg::X9
        );
        emit_ret!(insn);

        insn
    }

    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_sub_reg_imm!(insn, aarch64_reg::SP, aarch64_reg::SP, 16);
        emit_str_reg_imm!(insn, aarch64_reg::FP, aarch64_reg::SP, 0);
        emit_str_reg_imm!(insn, aarch64_reg::RA, aarch64_reg::SP, 8);
        emit_mov_reg_sp!(insn, aarch64_reg::FP, aarch64_reg::SP);
        emit_move_reg_imm!(insn, aarch64_reg::X9, fn_ptr as usize);
        emit_call_reg!(insn, aarch64_reg::X9);
        emit_ldr_reg_imm!(insn, aarch64_reg::FP, aarch64_reg::SP, 0);
        emit_ldr_reg_imm!(insn, aarch64_reg::RA, aarch64_reg::SP, 8);
        emit_add_reg_imm!(insn, aarch64_reg::SP, aarch64_reg::SP, 16);
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

    fn emit_usize_call_with_4_args(
        fn_ptr: extern "C" fn(usize, usize, usize, usize) -> usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
    ) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_sub_reg_imm!(insn, aarch64_reg::SP, aarch64_reg::SP, 16);
        emit_str_reg_imm!(insn, aarch64_reg::FP, aarch64_reg::SP, 0);
        emit_str_reg_imm!(insn, aarch64_reg::RA, aarch64_reg::SP, 8);
        emit_mov_reg_sp!(insn, aarch64_reg::FP, aarch64_reg::SP);
        emit_move_reg_imm!(insn, aarch64_abi::ARG0, arg1);
        emit_move_reg_imm!(insn, aarch64_abi::ARG1, arg2);
        emit_move_reg_imm!(insn, aarch64_abi::ARG2, arg3);
        emit_move_reg_imm!(insn, aarch64_abi::ARG3, arg4);
        emit_move_reg_imm!(insn, aarch64_reg::X9, fn_ptr as usize);
        emit_call_reg!(insn, aarch64_reg::X9);
        emit_ldr_reg_imm!(insn, aarch64_reg::FP, aarch64_reg::SP, 0);
        emit_ldr_reg_imm!(insn, aarch64_reg::RA, aarch64_reg::SP, 8);
        emit_add_reg_imm!(insn, aarch64_reg::SP, aarch64_reg::SP, 16);

        insn
    }

    fn emit_void_call_with_1_arg(fn_ptr: extern "C" fn(usize), arg1: usize) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_sub_reg_imm!(insn, aarch64_reg::SP, aarch64_reg::SP, 16);
        emit_str_reg_imm!(insn, aarch64_reg::FP, aarch64_reg::SP, 0);
        emit_str_reg_imm!(insn, aarch64_reg::RA, aarch64_reg::SP, 8);
        emit_mov_reg_sp!(insn, aarch64_reg::FP, aarch64_reg::SP);
        emit_move_reg_imm!(insn, aarch64_abi::ARG0, arg1);
        emit_move_reg_imm!(insn, aarch64_reg::X9, fn_ptr as usize);
        emit_call_reg!(insn, aarch64_reg::X9);
        emit_ldr_reg_imm!(insn, aarch64_reg::FP, aarch64_reg::SP, 0);
        emit_ldr_reg_imm!(insn, aarch64_reg::RA, aarch64_reg::SP, 8);
        emit_add_reg_imm!(insn, aarch64_reg::SP, aarch64_reg::SP, 16);

        insn
    }

    #[inline(never)]
    unsafe fn call_jit_ptr(jit_ptr: *mut u8) {
        asm!(
        "sub sp, sp, 8 * 8",       // Reserve space for 7 registers
        "stp x19, x20, [sp]",
        "stp x21, x22, [sp, #16]",
        "stp x23, x24, [sp, #32]",
        "str x30, [sp, #48]",      // Store return address onto the stack
        "blr {0}",
        "ldr x30, [sp, #48]",      // Load the return address back to x30
        "ldp x23, x24, [sp, #32]",
        "ldp x21, x22, [sp, #16]",
        "ldp x19, x20, [sp]",
        "add sp, sp, 8 * 8",

        in(reg) jit_ptr,
        );
    }
}
