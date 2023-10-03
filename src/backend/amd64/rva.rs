use crate::backend::common;
use common::DecodeRet;

pub struct RvaImpl;

impl common::Rva for RvaImpl {
    fn emit_lr_w(_rd: u8, _rs1: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_sc_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amoswap_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amoadd_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amoxor_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amoor_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amoand_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amomin_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amomax_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amominu_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }

    fn emit_amomaxu_w(_rd: u8, _rs1: u8, _rs2: u8, _aq: bool, _rl: bool) -> DecodeRet {
        todo!()
    }
}
