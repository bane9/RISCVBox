use crate::cpu::Cpu;
use crate::util::EncodedInsn;

#[derive(Debug)]
pub enum JitError {
    InvalidInstruction(u32),
    ReachedBlockBoundary,
    UnknownError,
}

pub type PtrT = *mut u8;
pub type HostInsnT = u8;
pub const HOST_INSN_MAX_SIZE: usize = 16; // TODO: check worst case later
pub type HostEncodedInsn = EncodedInsn<HostInsnT, HOST_INSN_MAX_SIZE>;
pub type DecodeRet = Result<HostEncodedInsn, JitError>;

pub trait BackendCore {
    fn fill_with_target_nop(ptr: PtrT, size: usize);
}

pub trait Rvi {
    fn emit_addi(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_add(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sub(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_slli(cpu: &mut Cpu, rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_slti(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_sltiu(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_xori(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_srli(cpu: &mut Cpu, rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_srai(cpu: &mut Cpu, rd: u8, rs1: u8, shamt: u8) -> DecodeRet;
    fn emit_ori(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_andi(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_xor(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_srl(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sra(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_or(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_and(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sll(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_slt(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_sltu(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;

    fn emit_lui(cpu: &mut Cpu, rd: u8, imm: i32) -> DecodeRet;
    fn emit_auipc(cpu: &mut Cpu, rd: u8, imm: i32) -> DecodeRet;
    fn emit_jal(cpu: &mut Cpu, rd: u8, imm: i32) -> DecodeRet;
    fn emit_jalr(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_beq(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bne(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_blt(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bge(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bltu(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_bgeu(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;

    fn emit_lb(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lh(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lw(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lbu(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;
    fn emit_lhu(cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> DecodeRet;

    fn emit_sb(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_sh(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;
    fn emit_sw(cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> DecodeRet;

    fn emit_fence(cpu: &mut Cpu, pred: u8, succ: u8) -> DecodeRet;
    fn emit_fence_i(cpu: &mut Cpu) -> DecodeRet;

    fn emit_ecall(cpu: &mut Cpu) -> DecodeRet;
    fn emit_ebreak(cpu: &mut Cpu) -> DecodeRet;
}

pub trait Rvm {
    fn emit_mul(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulh(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulhsu(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_mulhu(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;

    fn emit_div(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_divu(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_rem(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
    fn emit_remu(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet;
}

pub trait Rva {
    fn emit_lr_w(cpu: &mut Cpu, rd: u8, rs1: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_sc_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoswap_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoadd_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoxor_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoor_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amoand_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomin_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomax_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amominu_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;

    fn emit_amomaxu_w(cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8, aq: bool, rl: bool) -> DecodeRet;
}
