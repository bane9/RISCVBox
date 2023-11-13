use crate::cpu::{cpu, csr};
use cpu::{Exception, Interrupt};

use super::{csr::MppMode, CpuReg};

pub fn has_pending_interrupt() -> Option<Interrupt> {
    let cpu = cpu::get_cpu();

    match cpu.mode {
        csr::MppMode::Machine => {
            if !cpu.csr.read_bit_mstatus(csr::bits::MIE) {
                return None;
            }
        }
        csr::MppMode::Supervisor => {
            if !cpu.csr.read_bit_sstatus(csr::bits::SIE) {
                return None;
            }
        }
        csr::MppMode::User => {}
    }

    let mie = cpu.csr.read(csr::register::MIE);
    let mip = cpu.csr.read(csr::register::MIP);

    let pending = (mie & mip) as usize;

    if pending == 0 {
        return None;
    }

    if (pending & csr::bits::MEIP) != 0 {
        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::MEIP_BIT, false);

        return Some(Interrupt::MachineExternal);
    } else if (pending & csr::bits::MSIP) != 0 {
        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::MSIP_BIT, false);

        return Some(Interrupt::MachineSoftware);
    } else if (pending & csr::bits::MTIP) != 0 {
        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::MTIP_BIT, false);

        return Some(Interrupt::MachineTimer);
    } else if (pending & csr::bits::SEIP) != 0 {
        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::SSIP_BIT, false);

        return Some(Interrupt::SupervisorExternal);
    } else if (pending & csr::bits::SSIP) != 0 {
        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::SSIP_BIT, false);

        return Some(Interrupt::SupervisorSoftware);
    } else if (pending & csr::bits::STIP) != 0 {
        cpu.csr
            .write_bit(csr::register::MIP, csr::bits::STIP_BIT, false);

        return Some(Interrupt::SupervisorTimer);
    }

    None
}

pub fn handle_interrupt(int_val: Interrupt) {
    assert!(int_val != Interrupt::None);

    let cpu = cpu::get_cpu();

    let pc = cpu.pc;
    let mode = cpu.mode;

    let mideleg_flag = if int_val != Interrupt::MachineTimer {
        ((cpu.csr.read(csr::register::MIDELEG) >> int_val as usize) & 1) != 0
    } else {
        false
    };

    if mideleg_flag & (mode == MppMode::Supervisor || mode == MppMode::User) {
        cpu.mode = MppMode::Supervisor;

        let stvec_val = cpu.csr.read(csr::register::STVEC);
        let vt_offset = if stvec_val & 1 == 0 {
            0
        } else {
            int_val as CpuReg * 4
        };

        cpu.pc = (stvec_val & !1) + vt_offset;

        cpu.csr.write(csr::register::SEPC, pc & !1);
        cpu.csr
            .write_bit_sstatus(csr::bits::SPIE, cpu.csr.read_bit_sstatus(csr::bits::SIE));
        cpu.csr.write_bit_sstatus(csr::bits::SIE, false);
        cpu.csr.write_mpp_mode(mode);
    } else {
        cpu.mode = MppMode::Machine;

        let mtvec_val = cpu.csr.read(csr::register::MTVEC);
        let vt_offset = if mtvec_val & 1 == 0 {
            0
        } else {
            int_val as CpuReg * 4
        };

        cpu.pc = (mtvec_val & !1) + vt_offset;

        cpu.csr.write(csr::register::MEPC, pc & !1);
        cpu.csr
            .write_bit_mstatus(csr::bits::MPIE, cpu.csr.read_bit_mstatus(csr::bits::MIE));
        cpu.csr.write_bit_mstatus(csr::bits::MIE, false);
        cpu.csr.write_mpp_mode(mode);
    }
}

pub fn handle_exception() {
    let cpu = cpu::get_cpu();

    assert!(cpu.exception != Exception::None);

    let pc = cpu.pc;
    let mode = cpu.mode;

    let exc_val = cpu.exception.to_cpu_reg() as usize;

    let mideleg_flag = ((cpu.csr.read(csr::register::MIDELEG) >> exc_val as usize) & 1) != 0;

    if mideleg_flag & (mode == MppMode::Supervisor || mode == MppMode::User) {
        cpu.mode = MppMode::Supervisor;

        let stvec_val = cpu.csr.read(csr::register::STVEC);

        cpu.pc = stvec_val & !1;

        cpu.csr.write(csr::register::SEPC, pc & !1);
        cpu.csr
            .write_bit_sstatus(csr::bits::SPIE, cpu.csr.read_bit_sstatus(csr::bits::SIE));
        cpu.csr.write_bit_sstatus(csr::bits::SIE, false);
        cpu.csr.write_mpp_mode(mode);
    } else {
        cpu.mode = MppMode::Machine;

        let mtvec_val = cpu.csr.read(csr::register::MTVEC);

        cpu.pc = mtvec_val & !1;

        cpu.csr.write(csr::register::MEPC, pc & !1);
        cpu.csr
            .write_bit_mstatus(csr::bits::MPIE, cpu.csr.read_bit_mstatus(csr::bits::MIE));
        cpu.csr.write_bit_mstatus(csr::bits::MIE, false);
        cpu.csr.write_mpp_mode(mode);
    }
}
