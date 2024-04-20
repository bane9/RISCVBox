use std::io::{Read, Write};

use crate::{
    bus::bus::*,
    cpu::{self, Exception},
};

const UART_BASE_ADDRESS: u64 = 0x10000000;
const UART_IRQN: u64 = 10;

const RHR: u64 = 0;
const THR: u64 = 0;
const DLL: u64 = 0;

const IER: u64 = 1;
const DLM: u64 = 1;

const ISR: u64 = 2;
const FCR: u64 = 2;

const LCR: u64 = 3;
const MCR: u64 = 4;
const LSR: u64 = 5;
const MSR: u64 = 6;
const SCR: u64 = 7;

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

    read_thread: Option<std::thread::JoinHandle<()>>,
}

static mut READ_CHAR: std::sync::atomic::AtomicU8 = std::sync::atomic::AtomicU8::new(0);

fn read_thread() {
    loop {
        let mut input = [0u8];
        std::io::stdin().read(&mut input).unwrap();

        unsafe { READ_CHAR.store(input[0], std::sync::atomic::Ordering::Release) };
    }
}

extern "C" {
    #[link_name = "SetConsoleMode"]
    fn SetConsoleMode(
        handle: winapi::um::winnt::HANDLE,
        mode: winapi::shared::minwindef::DWORD,
    ) -> winapi::shared::minwindef::BOOL;
    #[link_name = "GetConsoleMode"]
    fn GetConsoleMode(
        handle: winapi::um::winnt::HANDLE,
        mode: *mut winapi::shared::minwindef::DWORD,
    ) -> winapi::shared::minwindef::BOOL;
}

impl Ns16550 {
    pub fn new() -> Self {
        let mut this = Self {
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

            read_thread: None,
        };

        this.read_thread = Some(std::thread::spawn(read_thread));

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

        match adj_addr as u64 {
            THR => {
                if (self.lsr & LSR_DR) != 0 {
                    self.lsr &= !LSR_DR;
                }
                Ok(self.val as BusType)
            }
            IER => return Ok(self.ier as BusType),
            ISR => return Ok(self.isr as BusType),
            LCR => return Ok(self.lcr as BusType),
            MCR => return Ok(self.mcr as BusType),
            LSR => return Ok(self.lsr as BusType),
            MSR => return Ok(self.msr as BusType),
            SCR => return Ok(self.scr as BusType),
            _ => Err(Exception::LoadAccessFault(addr)),
        }
    }

    fn store(&mut self, addr: BusType, data: BusType, _size: BusType) -> Result<(), Exception> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        match adj_addr as u64 {
            THR => {
                let c = data as u8 as char;
                if c.is_ascii() {
                    print!("{}", c);
                    let _ = std::io::stdout().flush();
                }
                // println!("printing u8: {} as char: {}", data as u8, c);
                if c == '\n' {
                    let _ = std::io::stdout().flush();
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
            _ => Err(Exception::StoreAccessFault(addr)),
        }
    }

    fn get_begin_addr(&self) -> BusType {
        UART_BASE_ADDRESS as BusType
    }

    fn get_end_addr(&self) -> BusType {
        (UART_BASE_ADDRESS + 8) as BusType
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {
        println!("tick_from_main_thread");
    }

    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        if dispatch_irq(self) {
            return Some(UART_IRQN as u32);
        }

        let c = unsafe { READ_CHAR.load(std::sync::atomic::Ordering::Acquire) };

        if c != 0 {
            self.lsr |= LSR_DR;
            self.val = c;
            unsafe { READ_CHAR.store(0, std::sync::atomic::Ordering::Release) };
        }

        if dispatch_irq(self) {
            Some(UART_IRQN as u32)
        } else {
            None
        }
    }
}
