use crate::{
    cpu::{
        cpu,
        csr::{self},
    },
    frontend::exec_core::INSN_SIZE,
};
use cpu::{Exception, Interrupt};

use super::{csr::MppMode, CpuReg};

pub fn are_interrupts_enabled(cpu: &mut cpu::Cpu) -> bool {
    match cpu.mode {
        csr::MppMode::Machine => {
            if !cpu.csr.read_bit_mstatus(csr::bits::MIE) {
                return false;
            }
        }
        csr::MppMode::Supervisor => {
            if !cpu.csr.read_bit_sstatus(csr::bits::SIE) {
                return false;
            }
        }
        csr::MppMode::User => {}
    }

    true
}

pub fn has_pending_interrupt(cpu: &mut cpu::Cpu) -> Option<Interrupt> {
    if !are_interrupts_enabled(cpu) {
        return None;
    }

    let mie = cpu.csr.read(csr::register::MIE);
    let mip = cpu.csr.load_mip_atomic();

    let pending = (mie & mip) as usize;

    if pending == 0 {
        return None;
    }

    let irq: Option<Interrupt>;

    if (pending & csr::bits::MEIP) != 0 {
        cpu.csr.clear_bit_mip_atomic(csr::bits::MEIP_BIT);

        irq = Some(Interrupt::MachineExternal);
    } else if (pending & csr::bits::MSIP) != 0 {
        cpu.csr.clear_bit_mip_atomic(csr::bits::MSIP_BIT);

        irq = Some(Interrupt::MachineSoftware);
    } else if (pending & csr::bits::MTIP) != 0 {
        cpu.csr.clear_bit_mip_atomic(csr::bits::MTIP_BIT);

        irq = Some(Interrupt::MachineTimer);
    } else if (pending & csr::bits::SEIP) != 0 {
        cpu.csr.clear_bit_mip_atomic(csr::bits::SEIP_BIT);

        irq = Some(Interrupt::SupervisorExternal);
    } else if (pending & csr::bits::SSIP) != 0 {
        cpu.csr.clear_bit_mip_atomic(csr::bits::SSIP_BIT);

        irq = Some(Interrupt::SupervisorSoftware);
    } else if (pending & csr::bits::STIP) != 0 {
        cpu.csr.clear_bit_mip_atomic(csr::bits::STIP_BIT);

        irq = Some(Interrupt::SupervisorTimer);
    } else {
        irq = None;
    }

    irq
}

pub fn handle_interrupt(int_val: Interrupt, cpu: &mut cpu::Cpu) {
    assert!(int_val != Interrupt::None);

    cpu.has_pending_interrupt
        .store(0, std::sync::atomic::Ordering::Release);

    let mode = cpu.mode;

    let mideleg_flag = if int_val != Interrupt::MachineTimer {
        ((cpu.csr.read(csr::register::MIDELEG) >> int_val as usize) & 1) != 0
    } else {
        false
    };

    let pc = cpu.next_pc as CpuReg;

    if mideleg_flag && (mode == MppMode::Supervisor || mode == MppMode::User) {
        cpu.mode = MppMode::Supervisor;

        let stvec_val = cpu.csr.read(csr::register::STVEC);
        let vt_offset = if stvec_val & 1 == 0 {
            0
        } else {
            int_val as CpuReg * INSN_SIZE as CpuReg
        };

        assert!(stvec_val != 0);

        cpu.next_pc = (stvec_val & !1) + vt_offset;

        cpu.csr.write(csr::register::SEPC, pc & !1);
        cpu.csr.write(
            csr::register::SCAUSE,
            int_val.to_cpu_reg() | (1 << (CpuReg::BITS - 1)),
        );
        cpu.csr.write(csr::register::STVAL, 0);
        cpu.csr
            .write_bit_sstatus(csr::bits::SPIE, cpu.csr.read_bit_sstatus(csr::bits::SIE));
        cpu.csr.write_bit_sstatus(csr::bits::SIE, false);

        if mode == MppMode::User {
            cpu.csr.write_bit_sstatus(csr::bits::SPP, false);
        } else {
            cpu.csr.write_bit_sstatus(csr::bits::SPP, true);
        }
    } else {
        cpu.mode = MppMode::Machine;

        let mtvec_val = cpu.csr.read(csr::register::MTVEC);
        let vt_offset = if mtvec_val & 1 == 0 {
            0
        } else {
            int_val as CpuReg * INSN_SIZE as CpuReg
        };

        assert!(mtvec_val != 0);

        cpu.next_pc = (mtvec_val & !1) + vt_offset;

        cpu.csr.write(csr::register::MEPC, pc & !1);
        cpu.csr.write(
            csr::register::MCAUSE,
            int_val.to_cpu_reg() | (1 << (CpuReg::BITS - 1)),
        );
        cpu.csr.write(csr::register::MTVAL, 0);
        cpu.csr
            .write_bit_mstatus(csr::bits::MPIE, cpu.csr.read_bit_mstatus(csr::bits::MIE));
        cpu.csr.write_bit_mstatus(csr::bits::MIE, false);
        cpu.csr.write_mpp_mode(mode);
    }
}

pub fn handle_exception(cpu: &mut cpu::Cpu) {
    assert!(cpu.exception < Exception::None);

    cpu.has_pending_interrupt
        .store(0, std::sync::atomic::Ordering::Release);

    let pc = cpu.c_exception_pc as CpuReg;
    let mode = cpu.mode;

    let exc_val = cpu.exception.to_cpu_reg() as usize;

    let medeleg_flag = ((cpu.csr.read(csr::register::MEDELEG) >> exc_val as usize) & 1) != 0;

    if medeleg_flag && (mode == MppMode::Supervisor || mode == MppMode::User) {
        cpu.mode = MppMode::Supervisor;

        let stvec_val = cpu.csr.read(csr::register::STVEC);

        cpu.next_pc = stvec_val & !1;

        cpu.csr.write(csr::register::SEPC, pc & !1);
        cpu.csr
            .write(csr::register::SCAUSE, cpu.exception.to_cpu_reg());
        cpu.csr
            .write(csr::register::STVAL, cpu.exception.get_data());
        cpu.csr
            .write_bit_sstatus(csr::bits::SPIE, cpu.csr.read_bit_sstatus(csr::bits::SIE));
        cpu.csr.write_bit_sstatus(csr::bits::SIE, false);

        if mode == MppMode::User {
            cpu.csr.write_bit_sstatus(csr::bits::SPP, false);
        } else {
            cpu.csr.write_bit_sstatus(csr::bits::SPP, true);
        }
    } else {
        cpu.mode = MppMode::Machine;

        let mtvec_val = cpu.csr.read(csr::register::MTVEC);

        cpu.next_pc = mtvec_val & !1;

        cpu.csr.write(csr::register::MEPC, pc & !1);
        cpu.csr
            .write(csr::register::MCAUSE, cpu.exception.to_cpu_reg());
        cpu.csr
            .write(csr::register::MTVAL, cpu.exception.get_data());
        cpu.csr
            .write_bit_mstatus(csr::bits::MPIE, cpu.csr.read_bit_mstatus(csr::bits::MIE));
        cpu.csr.write_bit_mstatus(csr::bits::MIE, false);
        cpu.csr.write_mpp_mode(mode);
    }

    assert!(cpu.next_pc != 0);
}
