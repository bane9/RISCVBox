use std::io::{Read, Write};

use crossbeam::queue::ArrayQueue;
use lazy_static::lazy_static;

use crate::{
    bus::bus::*,
    cpu::{self, csr, Exception},
    util,
};

use super::plic::PLIC_PHANDLE;

const UART_ADDR: BusType = 0x10000000;
const UART_SIZE: BusType = 10;
const UART_END_ADDR: BusType = UART_ADDR + UART_SIZE;
pub const UART_IRQN: BusType = 10;

const RHR: BusType = 0;
const THR: BusType = 0;
const DLL: BusType = 0;

const IER: BusType = 1;
const DLM: BusType = 1;

const ISR: BusType = 2;
const FCR: BusType = 2;

const LCR: BusType = 3;
const MCR: BusType = 4;
const LSR: BusType = 5;
const MSR: BusType = 6;
const SCR: BusType = 7;
const DOOM: BusType = 8;
const DOOM_FLUSH: BusType = 8;

const LSR_DR: u8 = 0x1;
const LSR_THRE: u8 = 0x20;
const LSR_TEMT: u8 = 0x40;

const IER_RDI: u8 = 0x01;
const IER_THRI: u8 = 0x02;

const LCR_DLAB: u8 = 0x80;

const ISR_NO_INT: u8 = 0x01;
const ISR_THRI: u8 = 0x02;
const ISR_RDI: u8 = 0x04;

pub struct Ns16550 {
    dll: u8,
    dlm: u8,
    isr: u8,
    ier: u8,
    fcr: u8,
    lcr: u8,
    mcr: u8,
    lsr: u8,
    msr: u8,
    scr: u8,
    val: u8,
    lol: bool,
}

lazy_static! {
    static ref CHARBUF: ArrayQueue<u8> = ArrayQueue::new(1024);
    static ref CHARBUF_KBD: ArrayQueue<u8> = ArrayQueue::new(8); // This is mainly meant for DOOM so it can detect key release events
}

pub fn write_char_cb(c: u8) {
    let _ = CHARBUF.push(c);
}

pub fn write_char_kbd(c: u8) {
    let _ = CHARBUF_KBD.push(c);
}

pub fn charbuf_read_data() -> Option<u8> {
    CHARBUF.pop()
}

fn charbuf_kbd_read_data() -> Option<u8> {
    CHARBUF_KBD.pop()
}

fn charkbd_flush() {
    while let Some(_) = CHARBUF_KBD.pop() {}
}

fn charbuf_has_data() -> bool {
    !CHARBUF.is_empty()
}

fn read_thread() {
    loop {
        static mut CTRL_A_PRESSED: bool = false;

        let mut input = [0u8];
        std::io::stdin().read(&mut input).unwrap();

        unsafe {
            if input[0] == 1 {
                CTRL_A_PRESSED = true;
                continue;
            } else if (input[0] == 'X' as u8 || input[0] == 'x' as u8) && CTRL_A_PRESSED {
                std::process::exit(0);
            } else {
                CTRL_A_PRESSED = false;
            }
        }

        write_char_cb(input[0]);
    }
}

fn stdout_flush_thread() {
    loop {
        std::io::stdout().flush().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
}

fn init_threads_once() {
    static INIT: std::sync::Once = std::sync::Once::new();

    INIT.call_once(|| {
        let _ = std::thread::spawn(read_thread);
        let _ = std::thread::spawn(stdout_flush_thread);
    });
}

impl Ns16550 {
    pub fn new() -> Self {
        let this = Self {
            dll: 0,
            dlm: 0,
            isr: 0,
            ier: 0,
            fcr: 0,
            lcr: 0,
            mcr: 0,
            lsr: LSR_TEMT | LSR_THRE,
            msr: 0,
            scr: 0,
            val: 0,
            lol: false,
        };

        init_threads_once();

        this
    }
}

fn dispatch_irq(uart: &mut Ns16550) -> bool {
    uart.isr |= 0xc0;

    if ((uart.ier & IER_RDI) != 0) && ((uart.lsr & LSR_DR) != 0) {
        return true;
    } else if ((uart.ier & IER_THRI) != 0) && ((uart.lsr & LSR_TEMT) != 0) {
        return true;
    }

    false
}

impl BusDevice for Ns16550 {
    fn load(&mut self, addr: BusType, _size: BusType) -> Result<BusType, Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        match adj_addr as BusType {
            THR => {
                let c = charbuf_read_data();

                let c = if let Some(c) = c { c } else { 0 };

                self.lsr &= !LSR_DR;

                Ok(c as BusType)
            }
            IER => return Ok(self.ier as BusType),
            ISR => return Ok(self.isr as BusType),
            LCR => return Ok(self.lcr as BusType),
            MCR => return Ok(self.mcr as BusType),
            LSR => return Ok(self.lsr as BusType),
            MSR => return Ok(self.msr as BusType),
            SCR => return Ok(self.scr as BusType),
            DOOM => {
                let c = charbuf_kbd_read_data();

                self.val = if let Some(c) = c { c } else { self.val };

                Ok(self.val as BusType)
            }

            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    fn store(&mut self, addr: BusType, data: BusType, _size: BusType) -> Result<(), Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        match adj_addr as BusType {
            THR => {
                let c = data as u8 as char;

                if c.is_ascii() && !self.lol {
                    std::io::stdout().write_all(&[c as u8]).unwrap();
                }

                Ok(())
            }
            IER => {
                self.ier = data as u8;
                Ok(())
            }
            FCR => {
                self.fcr = data as u8;
                Ok(())
            }
            LCR => {
                self.lcr = data as u8;
                Ok(())
            }
            MCR => {
                self.mcr = data as u8;
                Ok(())
            }
            SCR => {
                self.scr = data as u8;
                Ok(())
            }
            DOOM_FLUSH => {
                charkbd_flush();
                self.lol = true;

                Ok(())
            }
            _ => Err(Exception::StoreAccessFault(addr)),
        }
    }

    fn get_begin_addr(&self) -> BusType {
        UART_ADDR
    }

    fn get_end_addr(&self) -> BusType {
        UART_END_ADDR
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {
        println!("tick_from_main_thread");
    }

    fn tick_async(&mut self, cpu: &mut cpu::Cpu) -> Option<u32> {
        if charbuf_has_data() && !self.lol {
            self.lsr |= LSR_DR;
            cpu.pending_interrupt_number = UART_IRQN;

            return Some(csr::bits::SEIP_BIT as u32);
        }

        if dispatch_irq(self) {
            cpu.pending_interrupt_number = UART_IRQN;

            return Some(csr::bits::SEIP_BIT as u32);
        }

        None
    }

    fn describe_fdt(&self, fdt: &mut vm_fdt::FdtWriter) {
        let serial_node = fdt
            .begin_node(&util::fdt_node_addr_helper("serial", UART_ADDR))
            .unwrap();
        fdt.property_u32("interrupts", UART_IRQN).unwrap();
        fdt.property_u32("interrupt-parent", PLIC_PHANDLE).unwrap();
        fdt.property_u32("clock-frequency", 0x384000).unwrap();
        fdt.property_array_u32("reg", &[0x00, UART_ADDR, 0x00, UART_SIZE])
            .unwrap();
        fdt.property_string("compatible", "ns16550a").unwrap();
        fdt.end_node(serial_node).unwrap();
    }
}
