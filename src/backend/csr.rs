use common::DecodeRet;

use crate::backend::common;
use crate::backend::target::core::{BackendCore, BackendCoreImpl};
use crate::bus::mmu::Mmu;
use crate::bus::BusType;
use crate::cpu::csr::{self, CsrType, MppMode};
use crate::cpu::{self, CpuReg, Exception};

use super::{ReturnableHandler, ReturnableImpl};

pub struct CsrImpl;

type CsrHandler = fn(usize, usize) -> Result<usize, Exception>;

const CSR_REG_ACCESS_FLAG: usize = 1 << (usize::BITS - 1);

const CSRRW: usize = 0;
const CSRRS: usize = 1;
const CSRRC: usize = 2;
const CSRRWI: usize = 3;
const CSRRSI: usize = 4;
const CSRRCI: usize = 5;

fn csr_default_handler(csr_reg: usize, csr_val: usize) -> Result<usize, Exception> {
    let csr = csr::get_csr();

    csr.write(csr_reg, csr_val as u32);

    Ok(csr_val)
}

fn csr_satp_handler(csr_reg: usize, csr_val: usize) -> Result<usize, Exception> {
    let cpu = cpu::get_cpu();

    if cpu.csr.read_bit_mstatus(csr::bits::TVM) {
        // TODO: replace 0 with the correct value
        return Err(Exception::IllegalInstruction(0));
    }

    let val = csr_default_handler(csr_reg, csr_val);

    cpu.mmu.update(val.unwrap() as BusType);

    val
}

static mut CSR_HANDLERS: [CsrHandler; csr::CSR_COUNT] = [csr_default_handler; csr::CSR_COUNT];

pub fn init_backend_csr() {
    let &mut csr_handlers = unsafe { &mut CSR_HANDLERS };
}

extern "C" fn csr_handler_cb(csr_reg: usize, rd_rhs: usize, op: usize, pc: usize) {
    let cpu = cpu::get_cpu();

    let rd = (rd_rhs >> 8) & 0x1f;
    let rhs = rd_rhs & 0xff;

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

    let rd_val = unsafe { &CSR_HANDLERS }[csr_reg](csr_reg, new_csr_val);

    if rd_val.is_err() {
        cpu.set_exception(rd_val.err().unwrap(), pc as CpuReg);

        ReturnableImpl::throw();
    }

    if rd != 0 {
        cpu.regs[rd] = rd_val.unwrap() as CsrType;
    }
}

// Nothing is more permanent than a temporary solution
extern "C" fn mret_handler_cb(pc: usize) {
    let cpu = cpu::get_cpu();

    if cpu.mode != MppMode::Machine {
        let mret: u32 = 0x30200073;
        cpu.set_exception(Exception::IllegalInstruction(mret), pc as CpuReg);

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
        let sret: u32 = 0x10200073;
        cpu.set_exception(Exception::IllegalInstruction(sret), pc as CpuReg);

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

        let insn = BackendCoreImpl::emit_void_call_with_4_args(
            csr_handler_cb,
            csr,
            (rd << 8) | rs1,
            CSRRW,
            cpu::get_cpu().pc as usize,
        );

        Ok(insn)
    }

    fn emit_csrrs(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) | CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(
            csr_handler_cb,
            csr,
            (rd << 8) | rs1,
            CSRRS,
            cpu::get_cpu().pc as usize,
        );

        Ok(insn)
    }

    fn emit_csrrc(rd: u8, rs1: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = (rs1 as usize) | CSR_REG_ACCESS_FLAG;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(
            csr_handler_cb,
            csr,
            (rd << 8) | rs1,
            CSRRC,
            cpu::get_cpu().pc as usize,
        );

        Ok(insn)
    }

    fn emit_csrrwi(rd: u8, zimm: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = zimm as usize;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(
            csr_handler_cb,
            csr,
            (rd << 8) | rs1,
            CSRRWI,
            cpu::get_cpu().pc as usize,
        );

        Ok(insn)
    }

    fn emit_csrrsi(rd: u8, zimm: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = zimm as usize;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(
            csr_handler_cb,
            csr,
            (rd << 8) | rs1,
            CSRRSI,
            cpu::get_cpu().pc as usize,
        );

        Ok(insn)
    }

    fn emit_csrrci(rd: u8, zimm: u8, csr: u16) -> DecodeRet {
        let rd = rd as usize;
        let rs1 = zimm as usize;
        let csr = csr as usize;

        let insn = BackendCoreImpl::emit_void_call_with_4_args(
            csr_handler_cb,
            csr,
            (rd << 8) | rs1,
            CSRRCI,
            cpu::get_cpu().pc as usize,
        );

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
