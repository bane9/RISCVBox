use common::DecodeRet;

use crate::backend::common;
use crate::backend::target::core::{BackendCore, BackendCoreImpl};
use crate::bus::{self, BusType};
use crate::cpu::csr::{self, MppMode};
use crate::cpu::{self, CpuReg, Exception};
use crate::frontend::exec_core::INSN_SIZE;

use super::{ReturnableHandler, ReturnableImpl};

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
        CSRRW => val,
        CSRRS => csr_val | val,
        CSRRC => csr_val & !val,
        CSRRWI => val,
        CSRRSI => csr_val | val,
        CSRRCI => csr_val & !val,
        _ => panic!("Invalid CSR operation"),
    };

    let rd_val = CSR_HANDLERS[csr_reg](csr_reg, new_csr_val) as u32;

    if rd != 0 {
        cpu.regs[rd] = rd_val;
    }
}

// Nothing is more permanent than a temporary solution
extern "C" fn mret_handler_cb(pc: usize) {
    let cpu = cpu::get_cpu();

    if cpu.mode != MppMode::Machine {
        // Not sure if this or the altrenative methods are worse
        let insn = bus::get_bus()
            .fetch(pc as BusType, INSN_SIZE as BusType * 8)
            .unwrap();

        cpu.set_exception(Exception::IllegalInstruction(insn), pc as CpuReg);

        ReturnableImpl::throw();
    }

    cpu.pc = cpu.csr.read(csr::register::MEPC);
    cpu.mode = cpu.csr.read_mpp_mode();

    if cpu.mode != MppMode::Machine {
        cpu.csr.write_bit_mstatus(csr::bits::MPRV, false);
    }

    cpu.csr
        .write_bit_mstatus(csr::bits::MIE, cpu.csr.read_bit_mstatus(csr::bits::MPIE));

    cpu.csr.write_bit_mstatus(csr::bits::MPIE, true);
    cpu.csr.write_mpp_mode(MppMode::User);

    cpu.set_exception(Exception::Mret, pc as CpuReg);
}

extern "C" fn sret_handler_cb(pc: usize) {
    let cpu = cpu::get_cpu();

    if cpu.csr.read_bit_mstatus(csr::bits::TSR) || cpu.mode == MppMode::Machine {
        let insn = bus::get_bus()
            .fetch(pc as BusType, INSN_SIZE as BusType * 8)
            .unwrap();
        cpu.set_exception(Exception::IllegalInstruction(insn), pc as CpuReg);

        ReturnableImpl::throw();
    }

    cpu.pc = cpu.csr.read(csr::register::SEPC);
    cpu.mode = cpu.csr.read_mpp_mode();

    if cpu.mode == MppMode::User {
        cpu.csr.write_bit_mstatus(csr::bits::MPRV, false);
    }

    cpu.csr
        .write_bit_mstatus(csr::bits::SIE, cpu.csr.read_bit_mstatus(csr::bits::SPIE));

    cpu.csr.write_bit_mstatus(csr::bits::SPIE, true);
    cpu.csr.write_mpp_mode(MppMode::User);

    cpu.set_exception(Exception::Sret, pc as CpuReg);
}

impl common::Csr for CsrImpl {
    fn emit_csrrw(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) | CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRW);

        Ok(insn)
    }

    fn emit_csrrs(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) | CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(csr_handler_cb, csr, rd, rs1, CSRRS);

        Ok(insn)
    }

    fn emit_csrrc(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) | CSR_REG_ACCESS_FLAG;
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

    fn emit_ecall() -> DecodeRet {
        let cpu = cpu::get_cpu();

        // TODO: check at runtime
        match cpu.mode {
            MppMode::Machine => {
                let insn = BackendCoreImpl::emit_ret_with_exception(
                    Exception::EnvironmentCallFromMMode(cpu.pc),
                );

                Ok(insn)
            }
            MppMode::Supervisor => {
                let insn = BackendCoreImpl::emit_ret_with_exception(
                    Exception::EnvironmentCallFromSMode(cpu.pc),
                );

                Ok(insn)
            }
            MppMode::User => {
                let insn = BackendCoreImpl::emit_ret_with_exception(
                    Exception::EnvironmentCallFromUMode(cpu.pc),
                );

                Ok(insn)
            }
        }
    }

    fn emit_ebreak() -> DecodeRet {
        let insn = BackendCoreImpl::emit_ret_with_exception(Exception::Breakpoint);

        Ok(insn)
    }

    fn emit_sret() -> DecodeRet {
        let mut insn =
            BackendCoreImpl::emit_void_call_with_1_arg(sret_handler_cb, cpu::get_cpu().pc as usize);
        let ret = BackendCoreImpl::emit_ret();

        insn.push_slice(ret.iter().as_slice());

        Ok(insn)
    }

    fn emit_mret() -> DecodeRet {
        let mut insn =
            BackendCoreImpl::emit_void_call_with_1_arg(mret_handler_cb, cpu::get_cpu().pc as usize);
        let ret = BackendCoreImpl::emit_ret();

        insn.push_slice(ret.iter().as_slice());

        Ok(insn)
    }

    fn emit_wfi() -> DecodeRet {
        todo!()
    }
}
