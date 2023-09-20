use crate::backend::common;
use common::DecodeRet;

pub struct RvmImpl;

impl common::Rvm for RvmImpl {
    fn emit_mul(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulh(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulhsu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulhu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_div(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_divu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_rem(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_remu(rd: u8, rs1: u8, rs2: u8) -> DecodeRet {
        todo!()
    }
}
