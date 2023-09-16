use crate::backend::common;
use common::PtrT;

pub struct RvmImpl;

impl common::Rvm for RvmImpl {
    fn emit_mul(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_mulh(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_mulhsu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_mulhu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_div(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_divu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_rem(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }

    fn emit_remu(
        ptr: PtrT,
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
    ) -> Result<(), common::JitError> {
        todo!()
    }
}
