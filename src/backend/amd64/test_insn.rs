#[allow(unused_imports)]
use crate::backend::common::test_asm_common;
#[allow(unused_imports)]
use crate::backend::target::core::*;
use crate::*;

test_encoded_insn!(
    test_push_rax,
    |enc: &mut HostEncodedInsn| emit_push_reg!(enc, amd64_reg::RAX),
    [0x50]
);

test_encoded_insn!(
    test_push_r8,
    |enc: &mut HostEncodedInsn| emit_push_reg!(enc, amd64_reg::R8),
    [0x41, 0x50]
);

test_encoded_insn!(
    test_mov_rbp_rsp,
    |enc: &mut HostEncodedInsn| emit_mov_reg_reg1!(enc, amd64_reg::RBP, amd64_reg::RSP),
    [0x48, 0x89, 0xe5]
);

test_encoded_insn!(
    test_emit_pop_rbp,
    |enc: &mut HostEncodedInsn| emit_pop_reg!(enc, amd64_reg::RBP),
    [0x5d]
);

test_encoded_insn!(
    test_mov_rax_64b,
    |enc: &mut HostEncodedInsn| emit_move_reg_imm!(enc, amd64_reg::RAX, 10000000000000000000),
    [0x48, 0xb8, 0x00, 0x00, 0xe8, 0x89, 0x04, 0x23, 0xc7, 0x8a]
);

test_encoded_insn!(
    test_mov_r8_64b,
    |enc: &mut HostEncodedInsn| emit_move_reg_imm!(enc, amd64_reg::R8, 10000000000000000000),
    [0x49, 0xb8, 0x00, 0x00, 0xe8, 0x89, 0x04, 0x23, 0xc7, 0x8a]
);

test_encoded_insn!(
    test_call_rax,
    |enc: &mut HostEncodedInsn| emit_call_reg!(enc, amd64_reg::RAX),
    [0xff, 0xd0]
);

test_encoded_insn!(
    test_call_r8,
    |enc: &mut HostEncodedInsn| emit_call_reg!(enc, amd64_reg::R8),
    [0x41, 0xff, 0xd0]
);

test_encoded_insn!(test_nop, |enc: &mut HostEncodedInsn| emit_nop!(enc), [0x90]);

test_encoded_insn!(test_ret, |enc: &mut HostEncodedInsn| emit_ret!(enc), [0xc3]);

test_encoded_insn!(
    test_mov_qword_ptr_rax_1,
    |enc: &mut HostEncodedInsn| emit_mov_qword_ptr!(enc, amd64_reg::RAX, 1),
    [0x48, 0xc7, 0x00, 0x01, 0x00, 0x00, 0x00]
);

test_encoded_insn!(
    test_mov_qword_ptr_r8_1,
    |enc: &mut HostEncodedInsn| emit_mov_qword_ptr!(enc, amd64_reg::R8, 1),
    [0x49, 0xc7, 0x00, 0x01, 0x00, 0x00, 0x00]
);

test_encoded_insn!(
    test_mov_dword_ptr_rax_1,
    |enc: &mut HostEncodedInsn| emit_mov_dword_ptr_imm!(enc, amd64_reg::RAX, 1),
    [0xc7, 0x00, 0x01, 0x00, 0x00, 0x00]
);

test_encoded_insn!(
    test_mov_dword_ptr_r8_1,
    |enc: &mut HostEncodedInsn| emit_mov_dword_ptr_imm!(enc, amd64_reg::R8, 1),
    [0x41, 0xc7, 0x00, 0x01, 0x00, 0x00, 0x00]
);

test_encoded_insn!(
    test_mov_dword_ptr_rax_rax,
    |enc: &mut HostEncodedInsn| emit_mov_dword_ptr_reg!(enc, amd64_reg::RAX, amd64_reg::RAX),
    [0x89, 0x00]
);

test_encoded_insn!(
    test_shl_rax_1,
    |enc: &mut HostEncodedInsn| emit_shl_reg_imm!(enc, amd64_reg::RAX, 1),
    [0x48, 0xc1, 0xe0, 0x01]
);

test_encoded_insn!(
    test_shl_rax_2,
    |enc: &mut HostEncodedInsn| emit_shl_reg_imm!(enc, amd64_reg::RAX, 2),
    [0x48, 0xc1, 0xe0, 0x02]
);

test_encoded_insn!(
    test_shl_r9_2,
    |enc: &mut HostEncodedInsn| emit_shl_reg_imm!(enc, amd64_reg::R9, 2),
    [0x49, 0xc1, 0xe1, 0x02]
);

test_encoded_insn!(
    test_shr_rax_1,
    |enc: &mut HostEncodedInsn| emit_shr_reg_imm!(enc, amd64_reg::RAX, 1),
    [0x48, 0xc1, 0xe8, 0x01]
);

test_encoded_insn!(
    test_shr_rax_2,
    |enc: &mut HostEncodedInsn| emit_shr_reg_imm!(enc, amd64_reg::RAX, 2),
    [0x48, 0xc1, 0xe8, 0x02]
);

test_encoded_insn!(
    test_shr_r9_2,
    |enc: &mut HostEncodedInsn| emit_shr_reg_imm!(enc, amd64_reg::R9, 2),
    [0x49, 0xc1, 0xe9, 0x02]
);

test_encoded_insn!(
    test_add_rax_32b,
    |enc: &mut HostEncodedInsn| emit_add_reg_imm!(enc, amd64_reg::RAX, 1000000000),
    [0x48, 0x05, 0x00, 0xCA, 0x9A, 0x3B]
);

test_encoded_insn!(
    test_add_rbx_32b,
    |enc: &mut HostEncodedInsn| emit_add_reg_imm!(enc, amd64_reg::RBX, 1000000000),
    [0x48, 0x81, 0xC3, 0x00, 0xCA, 0x9A, 0x3B]
);

test_encoded_insn!(
    test_add_r8_32b,
    |enc: &mut HostEncodedInsn| emit_add_reg_imm!(enc, amd64_reg::R8, 1000000000),
    [0x49, 0x81, 0xC0, 0x00, 0xCA, 0x9A, 0x3B]
);

test_encoded_insn!(
    test_mov_cl,
    |enc: &mut HostEncodedInsn| emit_mov_cl_imm!(enc, 0xff),
    [0xb1, 0xff]
);

test_encoded_insn!(
    test_shr_reg_rbx_cl,
    |enc: &mut HostEncodedInsn| emit_shr_reg_cl!(enc, amd64_reg::RBX),
    [0x48, 0xD3, 0xEB]
);

test_encoded_insn!(
    test_shr_reg_r9_cl,
    |enc: &mut HostEncodedInsn| emit_shr_reg_cl!(enc, amd64_reg::R9),
    [0x49, 0xD3, 0xE9]
);

test_encoded_insn!(
    test_shl_reg_rbx_cl,
    |enc: &mut HostEncodedInsn| emit_shl_reg_cl!(enc, amd64_reg::RBX),
    [0x48, 0xD3, 0xE3]
);

test_encoded_insn!(
    test_shl_reg_r9_cl,
    |enc: &mut HostEncodedInsn| emit_shl_reg_cl!(enc, amd64_reg::R9),
    [0x49, 0xD3, 0xE1]
);

test_encoded_insn!(
    test_sub_rax_32b,
    |enc: &mut HostEncodedInsn| emit_sub_reg_imm!(enc, amd64_reg::RAX, 1000000000),
    [0x48, 0x2D, 0x00, 0xCA, 0x9A, 0x3B]
);

test_encoded_insn!(
    test_sub_rbx_32b,
    |enc: &mut HostEncodedInsn| emit_sub_reg_imm!(enc, amd64_reg::RBX, 1000000000),
    [0x48, 0x81, 0xEB, 0x00, 0xCA, 0x9A, 0x3B]
);

test_encoded_insn!(
    test_sub_r8_32b,
    |enc: &mut HostEncodedInsn| emit_sub_reg_imm!(enc, amd64_reg::R8, 1000000000),
    [0x49, 0x81, 0xE8, 0x00, 0xCA, 0x9A, 0x3B]
);

test_encoded_insn!(
    test_sub_rax_rax,
    |enc: &mut HostEncodedInsn| emit_sub_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RAX),
    [0x48, 0x29, 0xC0]
);

test_encoded_insn!(
    test_sub_rax_rcx,
    |enc: &mut HostEncodedInsn| emit_sub_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RCX),
    [0x48, 0x29, 0xC8]
);

test_encoded_insn!(
    test_sub_rcx_rax,
    |enc: &mut HostEncodedInsn| emit_sub_reg_reg!(enc, amd64_reg::RCX, amd64_reg::RAX),
    [0x48, 0x29, 0xC1]
);

test_encoded_insn!(
    test_xor_rax_imm,
    |enc: &mut HostEncodedInsn| emit_xor_reg_imm!(enc, amd64_reg::RAX, 0xaaaaaaa),
    [0x48, 0x35, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_xor_rbx_imm,
    |enc: &mut HostEncodedInsn| emit_xor_reg_imm!(enc, amd64_reg::RBX, 0xaaaaaaa),
    [0x48, 0x81, 0xF3, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_xor_r8_imm,
    |enc: &mut HostEncodedInsn| emit_xor_reg_imm!(enc, amd64_reg::R8, 0xaaaaaaa),
    [0x49, 0x81, 0xF0, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_xor_rax_rax,
    |enc: &mut HostEncodedInsn| emit_xor_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RAX),
    [0x48, 0x31, 0xC0]
);

test_encoded_insn!(
    test_xor_rax_rcx,
    |enc: &mut HostEncodedInsn| emit_xor_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RCX),
    [0x48, 0x31, 0xC8]
);

test_encoded_insn!(
    test_xor_rcx_rax,
    |enc: &mut HostEncodedInsn| emit_xor_reg_reg!(enc, amd64_reg::RCX, amd64_reg::RAX),
    [0x48, 0x31, 0xC1]
);

test_encoded_insn!(
    test_or_rax_imm,
    |enc: &mut HostEncodedInsn| emit_or_reg_imm!(enc, amd64_reg::RAX, 0xaaaaaaa),
    [0x48, 0x0D, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_or_rbx_imm,
    |enc: &mut HostEncodedInsn| emit_or_reg_imm!(enc, amd64_reg::RBX, 0xaaaaaaa),
    [0x48, 0x81, 0xCB, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_or_r8_imm,
    |enc: &mut HostEncodedInsn| emit_or_reg_imm!(enc, amd64_reg::R8, 0xaaaaaaa),
    [0x49, 0x81, 0xC8, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_or_rax_rax,
    |enc: &mut HostEncodedInsn| emit_or_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RAX),
    [0x48, 0x09, 0xC0]
);

test_encoded_insn!(
    test_or_rax_rcx,
    |enc: &mut HostEncodedInsn| emit_or_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RCX),
    [0x48, 0x09, 0xC8]
);

test_encoded_insn!(
    test_or_rcx_rax,
    |enc: &mut HostEncodedInsn| emit_or_reg_reg!(enc, amd64_reg::RCX, amd64_reg::RAX),
    [0x48, 0x09, 0xC1]
);

test_encoded_insn!(
    test_and_rax_imm,
    |enc: &mut HostEncodedInsn| emit_and_reg_imm!(enc, amd64_reg::RAX, 0xaaaaaaa),
    [0x48, 0x25, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_and_rbx_imm,
    |enc: &mut HostEncodedInsn| emit_and_reg_imm!(enc, amd64_reg::RBX, 0xaaaaaaa),
    [0x48, 0x81, 0xE3, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_and_r8_imm,
    |enc: &mut HostEncodedInsn| emit_and_reg_imm!(enc, amd64_reg::R8, 0xaaaaaaa),
    [0x49, 0x81, 0xE0, 0xAA, 0xAA, 0xAA, 0x0A]
);

test_encoded_insn!(
    test_and_rax_rax,
    |enc: &mut HostEncodedInsn| emit_and_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RAX),
    [0x48, 0x21, 0xC0]
);

test_encoded_insn!(
    test_and_rax_rcx,
    |enc: &mut HostEncodedInsn| emit_and_reg_reg!(enc, amd64_reg::RAX, amd64_reg::RCX),
    [0x48, 0x21, 0xC8]
);

test_encoded_insn!(
    test_and_rcx_rax,
    |enc: &mut HostEncodedInsn| emit_and_reg_reg!(enc, amd64_reg::RCX, amd64_reg::RAX),
    [0x48, 0x21, 0xC1]
);

test_encoded_insn!(
    test_jmp_rbx,
    |enc: &mut HostEncodedInsn| emit_jmp_reg!(enc, amd64_reg::RBX),
    [0xFF, 0xE3]
);

test_encoded_insn!(
    test_jmp_r9,
    |enc: &mut HostEncodedInsn| emit_jmp_reg!(enc, amd64_reg::R9),
    [0x41, 0xFF, 0xE1]
);
