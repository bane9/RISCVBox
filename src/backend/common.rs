use crate::cpu::Cpu;

#[derive(Debug)]
pub enum JitError {
    InvalidInstruction(u32),
    UnknownError,
}

pub type PtrT = *mut u8;

pub trait Rvi {
    fn emit_addi(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_add(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_sub(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_slli(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, shamt: u8) -> Result<(), JitError>;
    fn emit_slti(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_sltiu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_xori(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_srli(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, shamt: u8) -> Result<(), JitError>;
    fn emit_srai(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, shamt: u8) -> Result<(), JitError>;
    fn emit_ori(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_andi(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_xor(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_srl(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_sra(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_or(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_and(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_sll(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_slt(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_sltu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;

    fn emit_lui(ptr: PtrT, cpu: &mut Cpu, rd: u8, imm: i32) -> Result<(), JitError>;
    fn emit_auipc(ptr: PtrT, cpu: &mut Cpu, rd: u8, imm: i32) -> Result<(), JitError>;
    fn emit_jal(ptr: PtrT, cpu: &mut Cpu, rd: u8, imm: i32) -> Result<(), JitError>;
    fn emit_jalr(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_beq(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_bne(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_blt(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_bge(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_bltu(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_bgeu(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;

    fn emit_lb(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_lh(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_lw(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_lbu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;
    fn emit_lhu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, imm: i32) -> Result<(), JitError>;

    fn emit_sb(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_sh(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;
    fn emit_sw(ptr: PtrT, cpu: &mut Cpu, rs1: u8, rs2: u8, imm: i32) -> Result<(), JitError>;

    fn emit_fence(ptr: PtrT, cpu: &mut Cpu, pred: u8, succ: u8) -> Result<(), JitError>;
    fn emit_fence_i(ptr: PtrT, cpu: &mut Cpu) -> Result<(), JitError>;

    fn emit_ecall(ptr: PtrT, cpu: &mut Cpu) -> Result<(), JitError>;
    fn emit_ebreak(ptr: PtrT, cpu: &mut Cpu) -> Result<(), JitError>;
}

pub trait Rvm {
    fn emit_mul(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_mulh(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_mulhsu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_mulhu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;

    fn emit_div(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_divu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_rem(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
    fn emit_remu(ptr: PtrT, cpu: &mut Cpu, rd: u8, rs1: u8, rs2: u8) -> Result<(), JitError>;
}

pub trait Rva {
    fn emit_lr_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_sc_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;

    fn emit_amoswap_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amoadd_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amoxor_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amoor_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amoand_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amomin_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amomax_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amominu_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
    fn emit_amomaxu_w(
        ptr: PtrT,
        cpu: &mut Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> Result<(), JitError>;
}
