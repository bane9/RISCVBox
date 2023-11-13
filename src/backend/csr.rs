use common::DecodeRet;

use crate::backend::common;
use crate::backend::target::core::{BackendCore, BackendCoreImpl};
use crate::cpu;
use crate::cpu::csr;

pub struct CsrImpl;

type CsrHandler = fn(usize, usize) -> usize;

const CSR_REG_ACCESS_FLAG: usize = 1 << (usize::BITS - 1);

const CSRRW: usize = 0;
const CSRRS: usize = 1;
const CSRRC: usize = 2;
const CSRRWI: usize = 3;
const CSRRSI: usize = 4;
const CSRRCI: usize = 5;

fn csr_default_handler(csr_reg: usize, csr_val: usize) -> usize {
    let csr = csr::get_csr();

    csr.write(csr_reg, csr_val as u32);

    return csr_val;
}

const CSR_HANDLERS: [CsrHandler; csr::CSR_COUNT] = [csr_default_handler; csr::CSR_COUNT];

extern "C" fn csr_handler_cb(csr_reg: usize, rd: usize, rhs: usize, op: usize) {
    let cpu = cpu::get_cpu();

    let val: usize = if rhs & CSR_REG_ACCESS_FLAG != 0 {
        cpu.regs[rhs & !CSR_REG_ACCESS_FLAG] as usize
    } else {
        rhs
    };

    let csr_val = cpu.csr.read(csr_reg) as usize;

    let new_csr_val = match op {
        CSRRW => csr_val,
        CSRRS => csr_val | val,
        CSRRC => csr_val & !val,
        CSRRWI => val,
        CSRRSI => csr_val | val,
        CSRRCI => csr_val & !val,
        _ => panic!("Invalid CSR operation"),
    };

    cpu.regs[rd] = CSR_HANDLERS[csr_reg](csr_reg, new_csr_val) as u32;
}

impl common::Csr for CsrImpl {
    fn emit_csrrw(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) & CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRW);

        Ok(insn)
    }

    fn emit_csrrs(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) & CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRS);

        Ok(insn)
    }

    fn emit_csrrc(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) & CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRC);

        Ok(insn)
    }

    fn emit_csrrwi(rd: u8, zimm: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = zimm as usize;
        let csr = csr as usize;

        let insn =
            BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRWI);

        Ok(insn)
    }

    fn emit_csrrsi(rd: u8, zimm: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = zimm as usize;
        let csr = csr as usize;

        let insn =
            BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRSI);

        Ok(insn)
    }

    fn emit_csrrci(rd: u8, zimm: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = zimm as usize;
        let csr = csr as usize;

        let insn =
            BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRCI);

        Ok(insn)
    }
}
