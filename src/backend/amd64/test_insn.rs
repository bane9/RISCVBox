use crate::backend::amd64::core::{amd64_reg, HostEncodedInsn};
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
    [0x48, 0x05, 0x00, 0xca, 0x9a, 0x3b]
);

test_encoded_insn!(
    test_add_rbx_32b,
    |enc: &mut HostEncodedInsn| emit_add_reg_imm!(enc, amd64_reg::RAX, 1000000000),
    [0x48, 0x05, 0x00, 0xca, 0x9a, 0x3b]
);

test_encoded_insn!(
    test_add_r8_32b,
    |enc: &mut HostEncodedInsn| emit_add_reg_imm!(enc, amd64_reg::RAX, 1000000000),
    [0x48, 0x05, 0x00, 0xca, 0x9a, 0x3b]
);
