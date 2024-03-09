#![allow(unused_unsafe)]

use crate::{
    backend::*,
    bus::{self, BusType},
    frontend::exec_core::{RV_PAGE_MASK, RV_PAGE_OFFSET_MASK},
    xmem::{PageAllocator, PageState},
};
pub use crate::{cpu::*, util::EncodedInsn};

use crate::backend::amd64::rvi::emit_bus_access_raw;

use std::arch::asm;

pub type PtrT = *mut u8;
pub type HostInsnT = u8;
pub const HOST_INSN_MAX_SIZE: usize = 98;
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;

pub const FASTMEM_BLOCK_SIZE: usize = 88;

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

pub const MMU_IS_ACTIVE_REG: u8 = amd64_reg::R15;

#[cfg(windows)]
pub mod abi_reg {
    pub use super::amd64_reg;

    pub const ARG1: u8 = amd64_reg::RCX;
    pub const ARG2: u8 = amd64_reg::RDX;
    pub const ARG3: u8 = amd64_reg::R8;
    pub const ARG4: u8 = amd64_reg::R9;
}

#[cfg(unix)]
pub mod abi_reg {
    pub use super::amd64_reg;

    pub const ARG1: u8 = amd64_reg::RDI;
    pub const ARG2: u8 = amd64_reg::RSI;
    pub const ARG3: u8 = amd64_reg::RDX;
    pub const ARG4: u8 = amd64_reg::RCX;
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
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x50 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x41, 0x50 + $reg as u8 - amd64_reg::R8]);
        }
    }};
}

#[macro_export]
macro_rules! emit_mov_reg_reg1 {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!($enc, [0x48, 0x89, 0xC0 + ($src_reg << 3) + $dst_reg]);
    }};
}

#[macro_export]
macro_rules! emit_pop_reg {
    ($enc:expr, $reg:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x58 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x41, 0x58 + $reg as u8 - amd64_reg::R8]);
        }
    }};
}

#[macro_export]
macro_rules! emit_movabs_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xB8 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xB8 + $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, (($imm) as u64).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_reg_imm32 {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0xB8 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xC7, 0xC0 + $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, (($imm) as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_reg_imm_auto {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $imm as usize <= 0xFFFFFFFF {
            emit_mov_reg_imm32!($enc, $reg, $imm);
        } else {
            emit_movabs_reg_imm!($enc, $reg, $imm);
        }
    }};
}

#[macro_export]
macro_rules! emit_call_reg {
    ($enc:expr, $reg:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0xFF, 0xD0 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x41, 0xFF, 0xD0 + $reg as u8 - amd64_reg::R8]);
        }
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
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xC7, $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xC7, $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, (($imm) as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_dword_ptr_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0xC7, $reg as u8]);
        } else {
            emit_insn!($enc, [0x41, 0xC7, $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, (($imm) as u32).to_le_bytes());
    }};
}

// amd64 only supports < R8 for dword ptr
#[macro_export]
macro_rules! emit_mov_dword_ptr_reg {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x89,
                (0x00 as u8)
                    .wrapping_add($src_reg << 3)
                    .wrapping_add($dst_reg)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_mov_dword_ptr_reg_rip_relative {
    ($enc:expr, $src_reg:expr, $rip_offset:expr) => {{
        emit_insn!($enc, [0x89, (0x05 as u8).wrapping_add($src_reg << 3)]);
        emit_insn!($enc, ($rip_offset as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mov_ptr_reg_dword_ptr {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x8B,
                (0x00 as u8)
                    .wrapping_add($src_reg << 3)
                    .wrapping_add($dst_reg)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_mov_ptr_reg_dword_ptr_rip_relative {
    ($enc:expr, $dst_reg:expr, $rip_offset:expr) => {{
        assert!($dst_reg < amd64_reg::R8);
        emit_insn!($enc, [0x8B, (0x05 as u8).wrapping_add($dst_reg << 3)]);
        emit_insn!($enc, ($rip_offset as u32).to_le_bytes());
    }};
}

const INSN_MOV_RIP_RELATIVE_SIZE: usize = 6;

#[macro_export]
macro_rules! emit_mov_word_ptr_reg {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x66,
                0x89,
                (0x00 as u8)
                    .wrapping_add($src_reg << 3)
                    .wrapping_add($dst_reg)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_mov_ptr_reg_word_ptr {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x66,
                0x8B,
                (0x00 as u8)
                    .wrapping_add($src_reg << 3)
                    .wrapping_add($dst_reg)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_mov_byte_ptr_reg {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x88,
                (0x00 as u8)
                    .wrapping_add($src_reg << 3)
                    .wrapping_add($dst_reg)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_mov_ptr_reg_byte_ptr {
    ($enc:expr, $dst_reg:expr, $src_reg:expr) => {{
        assert!($dst_reg < amd64_reg::R8 && $src_reg < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x8A,
                (0x00 as u8)
                    .wrapping_add($src_reg << 3)
                    .wrapping_add($dst_reg)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_check_rd {
    ($enc:expr, $rd:expr) => {{
        if $rd == 0 {
            emit_nop!($enc);
            return Ok($enc);
        }
    }};
}

#[macro_export]
macro_rules! emit_set_exception {
    ($enc:expr, $cpu:expr, $exception:expr, $data:expr, $pc:expr) => {{
        let exception_addr = &$cpu.c_exception as *const _ as usize;
        emit_mov_reg_imm_auto!($enc, amd64_reg::RAX, exception_addr);
        emit_mov_dword_ptr_imm!($enc, amd64_reg::RAX, $exception as usize);

        let exception_data_addr = &$cpu.c_exception_data as *const _ as usize;
        emit_mov_reg_imm_auto!($enc, amd64_reg::RAX, exception_data_addr);
        emit_mov_dword_ptr_imm!($enc, amd64_reg::RAX, $data as usize);

        let exception_pc = &$cpu.c_exception_pc as *const _ as usize;
        emit_mov_reg_imm_auto!($enc, amd64_reg::RAX, exception_pc);
        emit_mov_dword_ptr_imm!($enc, amd64_reg::RAX, $pc as usize);

        emit_ret!($enc);
    }};
}

/////////// RVI

#[macro_export]
macro_rules! emit_shl_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xC1, 0xE0 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xC1, 0xE0 + $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, [$imm]);
    }};
}

#[macro_export]
macro_rules! emit_shr_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xC1, 0xE8 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xC1, 0xE8 + $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, [$imm]);
    }};
}

#[macro_export]
macro_rules! emit_mov_cl_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0xB1]);
        emit_insn!($enc, [$imm as u8]);
    }};
}

#[macro_export]
macro_rules! emit_movsxd_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x63,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_movsxd_reg64_reg8 {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x0f,
                0xbe,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_movsxd_reg64_reg16 {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x0f,
                0xbf,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_movzx_reg64_reg8 {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x0F,
                0xB6,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_movzx_reg64_reg16 {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x0F,
                0xB7,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_movsx_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x0F,
                0xBF,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_shr_reg_cl {
    ($enc:expr, $reg1:expr) => {{
        if $reg1 < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xD3, 0xE8 + $reg1 as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xD3, 0xE8 + $reg1 as u8 - amd64_reg::R8]);
        }
    }};
}

#[macro_export]
macro_rules! emit_sar_reg_cl {
    ($enc:expr, $reg1:expr) => {{
        if $reg1 < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xD3, 0xF8 + $reg1 as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xD3, 0xF8 + $reg1 as u8 - amd64_reg::R8]);
        }
    }};
}

#[macro_export]
macro_rules! emit_sarx_reg_reg {
    ($enc:expr, $dest_reg:expr, $reg1:expr, $reg2:expr) => {{
        assert!($dest_reg < amd64_reg::R8 && $reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0xC4,
                0xE2,
                0xf2,
                0xF7,
                (0xC0 as u8)
                    .wrapping_add($dest_reg << 3)
                    .wrapping_add($reg1),
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_shl_reg_cl {
    ($enc:expr, $reg1:expr) => {{
        if $reg1 < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0xD3, 0xE0 + $reg1 as u8]);
        } else {
            emit_insn!($enc, [0x49, 0xD3, 0xE0 + $reg1 as u8 - amd64_reg::R8]);
        }
    }};
}

#[macro_export]
macro_rules! emit_add_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg == amd64_reg::RAX {
            emit_insn!($enc, [0x05]);
        } else if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x81, 0xC0 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0x81, 0xC0 + $reg as u8 - amd64_reg::R8]);
        }

        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_sub_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg == amd64_reg::RAX {
            emit_insn!($enc, [0x2D]);
        } else if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x81, 0xE8 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0x81, 0xE8 + $reg as u8 - amd64_reg::R8]);
        }

        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_sub_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x29,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_add_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x01,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_xor_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg == amd64_reg::RAX {
            emit_insn!($enc, [0x48, 0x35]);
        } else if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0x81, 0xF0 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0x81, 0xF0 + $reg as u8 - amd64_reg::R8]);
        }
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_xor_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x31,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_or_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg == amd64_reg::RAX {
            emit_insn!($enc, [0x48, 0x0D]);
        } else if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0x81, 0xC8 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0x81, 0xC8 + $reg as u8 - amd64_reg::R8]);
        }

        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_or_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x09,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_and_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        if $reg == amd64_reg::RAX {
            emit_insn!($enc, [0x48, 0x25]);
        } else if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0x81, 0xE0 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x49, 0x81, 0xE0 + $reg as u8 - amd64_reg::R8]);
        }

        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_and_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x21,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_setl_al {
    ($enc:expr) => {{
        emit_insn!($enc, [0x0F, 0x9C, 0xC0]);
    }};
}

#[macro_export]
macro_rules! emit_setb_al {
    ($enc:expr) => {{
        emit_insn!($enc, [0x0F, 0x92, 0xC0]);
    }};
}

#[macro_export]
macro_rules! emit_setg_al {
    ($enc:expr) => {{
        emit_insn!($enc, [0x0F, 0x9F, 0xC0]);
    }};
}

#[macro_export]
macro_rules! emit_movzx_rax_al {
    ($enc:expr) => {{
        emit_insn!($enc, [0x48, 0x0F, 0xB6, 0xC0]);
    }};
}

#[macro_export]
macro_rules! emit_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x89,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_cmp_reg_imm {
    ($enc:expr, $reg1:expr, $imm:expr) => {{
        if $reg1 == amd64_reg::RAX {
            emit_insn!($enc, [0x48, 0x3D]);
        } else if $reg1 < amd64_reg::R8 {
            emit_insn!($enc, [0x48, 0x81, 0xF8 + $reg1 as u8]);
        } else {
            emit_insn!($enc, [0x49, 0x81, 0xF8 + ($reg1 as u8 - amd64_reg::R8)]);
        }

        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_cmp_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x39,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_cmp_reg_reg32 {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x39,
                (0xC0 as u8).wrapping_add($reg2 << 3).wrapping_add($reg1)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_cmp_reg_reg2 {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 >= amd64_reg::R8 && $reg2 >= amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x4D,
                0x39,
                (0xC0 as u8)
                    .wrapping_add(($reg2 - amd64_reg::R8) << 3)
                    .wrapping_add($reg1)
            ]
        );
    }};
}
// reg1 is always RAX
#[macro_export]
macro_rules! emit_test_less_reg_imm {
    ($enc:expr, $imm:expr) => {{
        emit_cmp_reg_imm!($enc, amd64_reg::RAX, $imm);
        emit_setl_al!($enc);
        emit_movzx_rax_al!($enc);
    }};
}

// reg1 is always RAX
#[macro_export]
macro_rules! emit_test_less_reg_uimm {
    ($enc:expr, $imm:expr) => {{
        emit_cmp_reg_imm!($enc, amd64_reg::RAX, $imm);
        emit_setb_al!($enc);
        emit_movzx_rax_al!($enc);
    }};
}

// reg1 is always RAX
#[macro_export]
macro_rules! emit_test_less_reg_reg {
    ($enc:expr, $reg2:expr) => {{
        emit_cmp_reg_reg!($enc, amd64_reg::RAX, $reg2);
        emit_setl_al!($enc);
        emit_movzx_rax_al!($enc);
    }};
}

// reg1 is always RAX
#[macro_export]
macro_rules! emit_test_greater_reg_imm {
    ($enc:expr, $imm:expr) => {{
        emit_cmp_reg_imm!($enc, amd64_reg::RAX, $imm);
        emit_setg_al!($enc);
        emit_movzx_rax_al!($enc);
    }};
}

#[macro_export]
macro_rules! emit_jz_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x84]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_jne_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x85]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_je_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x84]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_jl_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x8C]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_jb_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x82]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_jae_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x83]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_jge_imm {
    ($enc:expr, $imm:expr) => {{
        emit_insn!($enc, [0x0F, 0x8D]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

// reg1 is always RAX
#[macro_export]
macro_rules! emit_test_greater_reg_reg {
    ($enc:expr, $reg2:expr) => {{
        emit_cmp_reg_reg!($enc, amd64_reg::RAX, $reg2);
        emit_setg_al!($enc);
        emit_movzx_rax_al!($enc);
    }};
}

#[macro_export]
macro_rules! emit_jmp_reg {
    ($enc:expr, $reg:expr) => {{
        if $reg < amd64_reg::R8 {
            emit_insn!($enc, [0xFF, 0xE0 + $reg as u8]);
        } else {
            emit_insn!($enc, [0x41, 0xFF, 0xE0 + $reg as u8 - amd64_reg::R8]);
        }
    }};
}

#[macro_export]
macro_rules! emit_jmp_rip_relative_imm32 {
    ($enc:expr, $reg:expr) => {{
        emit_insn!($enc, [0xFF, 0x25]);
        emit_insn!($enc, ($reg as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_jmp_imm32 {
    ($enc:expr, $offset:expr) => {{
        emit_insn!($enc, [0xE9]);
        emit_insn!($enc, ($offset as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! get_jmp_imm32 {
    ($enc:expr, $reg:expr) => {{
        let mut imm = 0u32;

        unsafe {
            std::ptr::copy_nonoverlapping(
                $enc.as_ptr().wrapping_add($reg as usize).wrapping_add(1),
                &mut imm as *mut _ as *mut u8,
                std::mem::size_of::<u32>(),
            );
        }

        imm
    }};
}

#[macro_export]
macro_rules! patch_jmp_imm32 {
    ($insn_ptr:expr, $imm:expr) => {{
        let mut imm = $imm.to_le_bytes();
        unsafe {
            std::ptr::copy_nonoverlapping(
                &mut imm as *mut _ as *mut u8,
                $insn_ptr.wrapping_add(1),
                std::mem::size_of::<u32>(),
            );
        }
    }};
}

pub const JMP_IMM32_SIZE: usize = 5;
pub const CMP_JMP_IMM32_SIZE: usize = 6;

/////////// RVM

#[macro_export]
macro_rules! emit_mul_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        assert!($reg < amd64_reg::R8);
        emit_insn!($enc, [0x48, 0xF7, 0xC0 + $reg]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_mul_reg {
    ($enc:expr, $reg1:expr) => {{
        assert!($reg1 < amd64_reg::R8);
        emit_insn!($enc, [0x48, 0xF7, (0xE0 as u8).wrapping_add($reg1)]);
    }};
}

#[macro_export]
macro_rules! emit_div_reg_imm {
    ($enc:expr, $reg:expr, $imm:expr) => {{
        assert!($reg < amd64_reg::R8);
        emit_insn!($enc, [0x48, 0xF7, 0xF8 + $reg]);
        emit_insn!($enc, ($imm as u32).to_le_bytes());
    }};
}

#[macro_export]
macro_rules! emit_div_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0xF7,
                (0xF0 as u8).wrapping_add($reg1 << 3).wrapping_add($reg2)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_div_reg {
    ($enc:expr, $reg:expr) => {{
        assert!($reg < amd64_reg::R8);
        emit_insn!($enc, [0x48, 0xF7, (0xF8 as u8).wrapping_add($reg)]);
    }};
}

#[macro_export]
macro_rules! emit_div32_reg {
    ($enc:expr, $reg:expr) => {{
        assert!($reg < amd64_reg::R8);
        emit_insn!($enc, [0xF7, (0xF8 as u8).wrapping_add($reg)]);
    }};
}

#[macro_export]
macro_rules! emit_idiv_reg {
    ($enc:expr, $reg:expr) => {{
        assert!($reg < amd64_reg::R8);
        emit_insn!($enc, [0x48, 0xF7, (0xF8 as u8).wrapping_add($reg)]);
    }};
}

#[macro_export]
macro_rules! emit_idiv32_reg {
    ($enc:expr, $reg:expr) => {{
        assert!($reg < amd64_reg::R8);
        emit_insn!($enc, [0xF7, (0xF8 as u8).wrapping_add($reg)]);
    }};
}

#[macro_export]
macro_rules! emit_cqo {
    ($enc:expr) => {{
        emit_insn!($enc, [0x48, 0x99]);
    }};
}

#[macro_export]
macro_rules! emit_cdq {
    ($enc:expr) => {{
        emit_insn!($enc, [0x99]);
    }};
}

#[macro_export]
macro_rules! emit_idiv_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0xF7,
                (0xF8 as u8).wrapping_add($reg1 << 3).wrapping_add($reg2)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_imul_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x48,
                0x0F,
                0xAF,
                (0xC0 as u8).wrapping_add($reg1 << 3).wrapping_add($reg2)
            ]
        );
    }};
}

#[macro_export]
macro_rules! emit_imul32_reg_reg {
    ($enc:expr, $reg1:expr, $reg2:expr) => {{
        assert!($reg1 < amd64_reg::R8 && $reg2 < amd64_reg::R8);
        emit_insn!(
            $enc,
            [
                0x0F,
                0xAF,
                (0xC0 as u8).wrapping_add($reg1 << 3).wrapping_add($reg2)
            ]
        );
    }};
}

// Other

pub fn emit_rel_load(
    enc: &mut HostEncodedInsn,
    cpu: &mut Cpu,
    host_dst_reg: u8,
    guest_addr: *mut CpuReg,
) -> bool {
    let guest_dst_addr = guest_addr as i64;

    let current_rip =
        cpu.jit_current_ptr as i64 + enc.size() as i64 + INSN_MOV_RIP_RELATIVE_SIZE as i64;

    let offset = guest_dst_addr - current_rip;

    if offset >= i32::MIN as i64 && offset < i32::MAX as i64 {
        emit_mov_ptr_reg_dword_ptr_rip_relative!(enc, host_dst_reg, offset as i32);
        return true;
    }

    false
}

pub fn emit_mov_reg_guest_to_host(
    enc: &mut HostEncodedInsn,
    cpu: &mut Cpu,
    host_dst_reg: u8,
    guest_src_reg: u8,
) {
    if guest_src_reg == 0 {
        emit_xor_reg_reg!(enc, host_dst_reg, host_dst_reg);
        return;
    }

    let guest_src_addr = &cpu.regs[guest_src_reg as usize] as *const CpuReg;

    if emit_rel_load(enc, cpu, host_dst_reg, guest_src_addr as *mut CpuReg) {
        return;
    }

    emit_mov_reg_imm_auto!(enc, host_dst_reg, guest_src_addr);
    emit_mov_ptr_reg_dword_ptr!(enc, host_dst_reg, host_dst_reg);
}

pub fn emit_rel_store(
    enc: &mut HostEncodedInsn,
    cpu: &mut Cpu,
    host_val_reg: u8,
    guest_dst_reg: *mut CpuReg,
) -> bool {
    let guest_dst_addr = guest_dst_reg as i64;

    let current_rip =
        cpu.jit_current_ptr as i64 + enc.size() as i64 + INSN_MOV_RIP_RELATIVE_SIZE as i64;

    let offset = guest_dst_addr - current_rip;

    if offset >= i32::MIN as i64 && offset < i32::MAX as i64 {
        emit_mov_dword_ptr_reg_rip_relative!(enc, host_val_reg, offset as i32);
        return true;
    }

    false
}

pub fn emit_mov_reg_host_to_guest(
    enc: &mut HostEncodedInsn,
    cpu: &mut Cpu,
    host_clobber_reg: u8,
    host_val_reg: u8,
    guest_dst_reg: u8,
) {
    if guest_dst_reg == 0 {
        emit_nop!(enc);
        return;
    }

    let guest_dst_addr = &cpu.regs[guest_dst_reg as usize] as *const CpuReg;

    if emit_rel_store(enc, cpu, host_val_reg, guest_dst_addr as *mut CpuReg) {
        return;
    }

    emit_mov_reg_imm_auto!(enc, host_clobber_reg, guest_dst_addr);
    emit_mov_dword_ptr_reg!(enc, host_clobber_reg, host_val_reg);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FastmemAccessType {
    Store = 0,
    Load = 1,
    LoadUnsigned = 2,
}

impl FastmemAccessType {
    pub fn from_usize(val: usize) -> Self {
        match val {
            0 => Self::Store,
            1 => Self::Load,
            2 => Self::LoadUnsigned,
            _ => panic!("Invalid fastmem access type"),
        }
    }

    pub fn to_usize(&self) -> usize {
        match self {
            Self::Store => 0,
            Self::Load => 1,
            Self::LoadUnsigned => 2,
        }
    }
}

#[macro_export]
macro_rules! extract_imm_from_movabs {
    ($ptr: expr) => {{
        let mut imm = 0u64;

        unsafe {
            std::ptr::copy_nonoverlapping(
                $ptr.wrapping_add(2),
                &mut imm as *mut _ as *mut u8,
                std::mem::size_of::<u64>(),
            );
        }

        imm
    }};
}

#[macro_export]
macro_rules! extract_imm_from_mov32 {
    ($ptr: expr) => {{
        let mut imm = 0u32;

        unsafe {
            std::ptr::copy_nonoverlapping(
                $ptr.wrapping_add(1),
                &mut imm as *mut _ as *mut u8,
                std::mem::size_of::<u32>(),
            );
        }

        imm
    }};
}

#[macro_export]
macro_rules! extract_imm_from_mov_auto {
    ($ptr: expr) => {{
        unsafe {
            if *$ptr == 0x48 || *$ptr == 0x49 {
                (extract_imm_from_movabs!($ptr), 10usize)
            } else if *$ptr >= 0xB8 && *$ptr <= 0xBF {
                (extract_imm_from_mov32!($ptr) as u64, 5usize)
            } else {
                panic!("Invalid mov instruction");
            }
        }
    }};
}

#[macro_export]
macro_rules! extract_fastmem_metadata {
    ($metadata: expr) => {{
        let access_size = ($metadata >> 8) & 0xFF;
        let access_type = $metadata & 0xFF;

        (access_size, access_type)
    }};
}

#[macro_export]
macro_rules! create_fastmem_metadata {
    ($access_size:expr, $access_type:expr) => {{
        (($access_size & 0xFF) << 8) | ($access_type & 0xFF)
    }};
}

pub struct BackendCoreImpl;

impl BackendCore for BackendCoreImpl {
    fn emit_atomic_access(mut insn: crate::backend::HostEncodedInsn) -> HostEncodedInsn {
        let ret_insn = Self::emit_ret();

        emit_cmp_reg_imm!(insn, amd64_reg::RAX, 0);
        emit_jz_imm!(insn, ret_insn.size());
        insn.push_slice(ret_insn.as_slice());

        insn
    }

    fn emit_ret() -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_ret!(insn);

        insn
    }

    fn emit_nop() -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_nop!(insn);

        insn
    }

    fn emit_ret_with_exception(exception: Exception) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        let exc_int = exception.to_cpu_reg() as usize;
        let exc_data = exception.get_data() as usize;

        let cpu = cpu::get_cpu();

        emit_set_exception!(insn, cpu, exc_int, exc_data, cpu.current_gpfn_offset);

        insn
    }

    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_push_reg!(insn, amd64_reg::RBP);
        emit_mov_reg_reg1!(insn, amd64_reg::RBP, amd64_reg::RSP);
        emit_mov_reg_imm_auto!(insn, amd64_reg::R11, fn_ptr);
        emit_call_reg!(insn, amd64_reg::R11);
        emit_pop_reg!(insn, amd64_reg::RBP);

        insn
    }

    fn emit_usize_call_with_4_args(
        fn_ptr: extern "C" fn(usize, usize, usize, usize) -> usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
    ) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_push_reg!(insn, amd64_reg::RBP);
        emit_mov_reg_reg1!(insn, amd64_reg::RBP, amd64_reg::RSP);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG1, arg1);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG2, arg2);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG3, arg3);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG4, arg4);
        emit_mov_reg_imm_auto!(insn, amd64_reg::R11, fn_ptr);
        emit_call_reg!(insn, amd64_reg::R11);
        emit_pop_reg!(insn, amd64_reg::RBP);

        insn
    }

    fn emit_usize_call_with_2_args(
        fn_ptr: extern "C" fn(usize, usize) -> usize,
        arg1: usize,
        arg2: usize,
    ) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_push_reg!(insn, amd64_reg::RBP);
        emit_mov_reg_reg1!(insn, amd64_reg::RBP, amd64_reg::RSP);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG1, arg1);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG2, arg2);
        emit_mov_reg_imm_auto!(insn, amd64_reg::R11, fn_ptr);
        emit_call_reg!(insn, amd64_reg::R11);
        emit_pop_reg!(insn, amd64_reg::RBP);

        insn
    }

    fn emit_void_call_with_2_args(
        fn_ptr: extern "C" fn(usize, usize),
        arg1: usize,
        arg2: usize,
    ) -> HostEncodedInsn {
        let fn_ptr =
            unsafe { std::mem::transmute::<_, extern "C" fn(usize, usize) -> usize>(fn_ptr) };

        Self::emit_usize_call_with_2_args(fn_ptr, arg1, arg2)
    }

    fn emit_void_call_with_4_args(
        fn_ptr: extern "C" fn(usize, usize, usize, usize),
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
    ) -> HostEncodedInsn {
        let fn_ptr = unsafe {
            std::mem::transmute::<_, extern "C" fn(usize, usize, usize, usize) -> usize>(fn_ptr)
        };

        Self::emit_usize_call_with_4_args(fn_ptr, arg1, arg2, arg3, arg4)
    }

    fn emit_void_call_with_1_arg(fn_ptr: extern "C" fn(usize), arg1: usize) -> HostEncodedInsn {
        let mut insn = HostEncodedInsn::new();

        emit_push_reg!(insn, amd64_reg::RBP);
        emit_mov_reg_reg1!(insn, amd64_reg::RBP, amd64_reg::RSP);
        emit_mov_reg_imm_auto!(insn, abi_reg::ARG1, arg1);
        emit_mov_reg_imm_auto!(insn, amd64_reg::R11, fn_ptr);
        emit_call_reg!(insn, amd64_reg::R11);
        emit_pop_reg!(insn, amd64_reg::RBP);

        insn
    }

    fn emit_usize_call_with_1_arg(
        fn_ptr: extern "C" fn(usize) -> usize,
        arg1: usize,
    ) -> HostEncodedInsn {
        let fn_ptr = unsafe { std::mem::transmute::<_, extern "C" fn(usize)>(fn_ptr) };

        Self::emit_void_call_with_1_arg(fn_ptr, arg1)
    }

    fn fastmem_violation_likely_offset() -> usize {
        FASTMEM_BLOCK_SIZE - 16
    }

    fn patch_fastmem_violation(
        host_exception_addr: usize,
        guest_exception_addr: BusType,
    ) -> FastmemHandleType {
        let host_insn_begin = host_exception_addr as *mut u8;

        let imm = extract_imm_from_mov_auto!(host_insn_begin);

        let (access_size, access_type) = extract_fastmem_metadata!(imm.0);

        let access_type = FastmemAccessType::from_usize(access_type as usize);

        let mut offset = imm.1;

        let reg1 = extract_imm_from_mov_auto!(host_insn_begin.wrapping_add(offset));

        offset += reg1.1;

        let reg2 = extract_imm_from_mov_auto!(host_insn_begin.wrapping_add(offset));

        offset += reg2.1;

        let imm = extract_imm_from_mov_auto!(host_insn_begin.wrapping_add(offset)).0 as i32;

        let reg1 = reg1.0 as *mut u8;
        let reg2 = reg2.0 as *mut u8;

        let gpfn_offset = guest_exception_addr as usize & RV_PAGE_OFFSET_MASK;
        let cpu = cpu::get_cpu();

        if access_type == FastmemAccessType::Store {
            let dst_addr =
                unsafe { std::ptr::read_unaligned(reg1 as *const CpuReg) as i64 + imm as i64 };

            let gpfn_state = cpu
                .gpfn_state
                .get_gpfn_state_mut(dst_addr as CpuReg & RV_PAGE_MASK as CpuReg);

            if gpfn_state.is_some() {
                let gpfn_state = gpfn_state.unwrap();

                if gpfn_state.get_state() == PageState::ReadExecute {
                    let src_val = unsafe { std::ptr::read_unaligned(reg2 as *const CpuReg) };

                    let bus = bus::get_bus();

                    gpfn_state.set_state(PageState::ReadWrite);

                    bus.store(
                        dst_addr as BusType,
                        src_val as BusType,
                        access_size as BusType,
                        &cpu.mmu,
                    )
                    .expect("Bus error while manually handling fastmem violation");

                    gpfn_state.set_state(PageState::ReadExecute);

                    return FastmemHandleType::Manual;
                }
            }
        }

        let insn = match access_type {
            FastmemAccessType::Store => match access_size {
                8 => emit_bus_access_raw(c_sb_cb, reg1, reg2, imm, gpfn_offset),
                16 => emit_bus_access_raw(c_sh_cb, reg1, reg2, imm, gpfn_offset),
                32 => emit_bus_access_raw(c_sw_cb, reg1, reg2, imm, gpfn_offset),
                _ => unreachable!(),
            },
            FastmemAccessType::Load => match access_size {
                8 => emit_bus_access_raw(c_lb_cb, reg1, reg2, imm, gpfn_offset),
                16 => emit_bus_access_raw(c_lh_cb, reg1, reg2, imm, gpfn_offset),
                32 => emit_bus_access_raw(c_lw_cb, reg1, reg2, imm, gpfn_offset),
                _ => unreachable!(),
            },
            FastmemAccessType::LoadUnsigned => match access_size {
                8 => emit_bus_access_raw(c_lbu_cb, reg1, reg2, imm, gpfn_offset),
                16 => emit_bus_access_raw(c_lhu_cb, reg1, reg2, imm, gpfn_offset),
                _ => unreachable!(),
            },
        };

        let page_size_mask = PageAllocator::get_page_size() - 1;
        let host_insn_begin_aligned = host_insn_begin as usize & !page_size_mask;
        let host_insn_begin_npages =
            (insn.size() + page_size_mask) / PageAllocator::get_page_size();

        PageAllocator::mark_page(
            host_insn_begin_aligned as *mut u8,
            host_insn_begin_npages,
            PageState::ReadWrite,
        )
        .expect("Failed to mark page as read-write while patching fastmem violation");

        unsafe {
            std::ptr::copy_nonoverlapping(insn.as_slice().as_ptr(), host_insn_begin, insn.size());
        }

        assert!(insn.size() <= FASTMEM_BLOCK_SIZE);

        let remaining_bytes = FASTMEM_BLOCK_SIZE - insn.size();

        unsafe {
            for i in 0..remaining_bytes {
                *host_insn_begin.wrapping_add(insn.size() + i) = 0x90; // nop
            }
        }

        PageAllocator::mark_page(
            host_insn_begin_aligned as *mut u8,
            host_insn_begin_npages,
            PageState::ReadExecute,
        )
        .expect("Failed to mark page as read-execute while patching fastmem violation");

        FastmemHandleType::Patched
    }

    fn patch_jump_list(jump_list: &Vec<JumpAddrPatch>) {
        let cpu = cpu::get_cpu();

        for patch in jump_list {
            let (guest_addr, host_addr, jmp_insn_offset) = patch.get_data();

            let host_target = cpu
                .insn_map
                .get_by_guest_idx(guest_addr)
                .expect("While patching relocation: guest address not found in insn map")
                .host_ptr;

            let diff = host_target as i64 - host_addr as i64 - jmp_insn_offset as i64;

            assert!(diff >= std::i32::MIN as i64 && diff <= std::i32::MAX as i64);

            let mut diff = (diff as u32).to_le_bytes();
            unsafe {
                std::ptr::copy_nonoverlapping(
                    &mut diff as *mut _ as *mut u8,
                    host_addr.wrapping_add(jmp_insn_offset as usize - std::mem::size_of::<u32>()),
                    std::mem::size_of::<u32>(),
                );
            }
        }
    }

    #[inline(never)]
    unsafe fn call_jit_ptr(jit_ptr: *mut u8) {
        asm!(
            "push rbp",
            "push rbx",
            "push rdi",
            "push rsi",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov r15, 1",
            "call {0}",

            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rsi",
            "pop rdi",
            "pop rbx",
            "pop rbp",

            in(reg) jit_ptr,
        );
    }

    #[inline(never)]
    unsafe fn call_jit_ptr_nommu(jit_ptr: *mut u8) {
        asm!(
            "push rbp",
            "push rbx",
            "push rdi",
            "push rsi",
            "push r12",
            "push r13",
            "push r14",
            "push r15",
            "mov r15, 0",
            "call {0}",

            "pop r15",
            "pop r14",
            "pop r13",
            "pop r12",
            "pop rsi",
            "pop rdi",
            "pop rbx",
            "pop rbp",

            in(reg) jit_ptr,
        );
    }
}
