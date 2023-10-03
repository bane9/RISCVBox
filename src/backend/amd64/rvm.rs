use crate::backend::common;
use common::DecodeRet;

pub struct RvmImpl;

impl common::Rvm for RvmImpl {
    fn emit_mul(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulh(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulhsu(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_mulhu(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_div(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_divu(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_rem(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }

    fn emit_remu(_rd: u8, _rs1: u8, _rs2: u8) -> DecodeRet {
        todo!()
    }
}
