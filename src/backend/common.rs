use crate::cpu::RunState;
use crate::util::EncodedInsn;

#[derive(Debug)]
pub enum JitError {
    InvalidInstruction(u32),
    ReachedBlockBoundary,
    UnknownError,
}

pub type PtrT = *mut u8;
pub type HostInsnT = u8;
pub const HOST_INSN_MAX_SIZE: usize = 64; // TODO: check worst case later
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;

#[macro_export]
macro_rules! test_encoded_insn {
    ($test_name:ident, $insn_macro:expr, $expected:expr) => {
        #[test]
        pub fn $test_name() {
            let mut enc = HostEncodedInsn::new();

            $insn_macro(&mut enc);

            let mut success = true;
            let mut expected_str = String::new();
            let mut encoded_str = String::new();

            expected_str.push_str("");
            encoded_str.push_str("");

            for (_, (a, b)) in enc.iter().zip($expected.iter()).enumerate() {
                if a != b {
                    success = false;

                    expected_str.push_str(&format!("\x1b[32m{:02x}\x1b[0m ", b));
                    encoded_str.push_str(&format!("\x1b[31m{:02x}\x1b[0m ", a));
                } else {
                    expected_str.push_str(&format!("{:02x} ", b));
                    encoded_str.push_str(&format!("{:02x} ", a));
                }
            }

            for &b in $expected.get(enc.size()..).unwrap_or(&[]) {
                success = false;
                expected_str.push_str(&format!("\x1b[32m{:02x}\x1b[0m ", b));
            }

            for &a in enc.iter().skip($expected.len()) {
                success = false;
                encoded_str.push_str(&format!("\x1b[31m{:02x}\x1b[0m ", a));
            }

            if !success {
                println!("__________________________________________________________\n");
                println!(
                    "Error: Encoding mismatch at \x1b[33m{}\x1b[0m",
                    &stringify!($insn_macro)[28..].trim().replace(" :: ", "::")
                );

                println!("Expected -> {}", expected_str.trim_end());
                println!("Encoded  -> {}", encoded_str.trim_end());
                println!("__________________________________________________________\n");
            }

            assert!(success);
        }
    };
}

pub trait BackendCore {
    fn fill_with_target_nop(ptr: PtrT, size: usize);
    fn fill_with_target_ret(ptr: PtrT, size: usize);
    fn emit_ret_with_status(state: RunState) -> HostEncodedInsn;
    fn emit_void_call(fn_ptr: extern "C" fn()) -> HostEncodedInsn;
    fn find_guest_pc_from_host_stack_frame(caller_ret_addr: *mut u8) -> Option<u32>;
    fn emit_usize_call_with_4_args(
        fn_ptr: extern "C" fn(usize, usize, usize, usize) -> usize,
        arg1: usize,
        arg2: usize,
        arg3: usize,
        arg4: usize,
    ) -> HostEncodedInsn;
    fn emit_void_call_with_1_arg(fn_ptr: extern "C" fn(usize), arg1: usize) -> HostEncodedInsn;
    unsafe fn call_jit_ptr(jit_ptr: PtrT);
}

pub trait Rvi {
    fn emit_addi(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_add(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sub(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_slli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_slti(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_sltiu(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_xori(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_srli(rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_srai(rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_ori(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_andi(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_xor(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_srl(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sra(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_or(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_and(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sll(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_slt(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sltu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;

    fn emit_lui(rd: u8, imm: i32) -> DecodeRet;
    fn emit_auipc(rd: u8, imm: i32) -> DecodeRet;
    fn emit_jal(rd: u8, imm: i32) -> DecodeRet;
    fn emit_jalr(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_beq(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bne(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_blt(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bge(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bltu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bgeu(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;

    fn emit_lb(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lh(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lw(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lbu(rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lhu(rd: u8, rs1: u8, imm: i32) -> DecodeRet;

    fn emit_sb(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_sh(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_sw(rs1: u8, rs2: u8, imm: i32) -> DecodeRet;

    fn emit_fence(pred: u8, succ: u8) -> DecodeRet;
    fn emit_fence_i() -> DecodeRet;

    fn emit_ecall() -> DecodeRet;
    fn emit_ebreak() -> DecodeRet;
}

pub trait Rvm {
    fn emit_mul(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulh(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulhsu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulhu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;

    fn emit_div(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_divu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_rem(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_remu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
}

pub trait Rva {
    fn emit_lr_w(rd: u8, rs1: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_sc_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoswap_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoadd_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoxor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoor_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoand_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomin_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomax_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amominu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomaxu_w(rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;
}
