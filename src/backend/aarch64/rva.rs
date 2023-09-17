use crate::backend::common;
use common::DecodeRet;

pub struct RvaImpl;

impl common::Rva for RvaImpl {
    fn emit_lr_w(cpu: &mut crate::cpu::Cpu, rd: u8, rs1: u8, aq: bool, rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_sc_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amoswap_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amoadd_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amoxor_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amoor_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amoand_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amomin_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amomax_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amominu_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }

    fn emit_amomaxu_w(
        cpu: &mut crate::cpu::Cpu,
        rd: u8,
        rs1: u8,
        rs2: u8,
        aq: bool,
        rl: bool,
    ) -> DecodeRet {
        todo!()
    }
}
