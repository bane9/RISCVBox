use crate::backend::common;
use common::DecodeRet;

pub struct RvmImpl;

impl common::Rvm for RvmImpl {
    fn emit_mul(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulh(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulhsu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulhu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_div(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_divu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_rem(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_remu(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }
}
