use crate::bus::bus::*;

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
}

impl Ns16550 {
    pub fn new() -> Self {
        Self {
            dll: 0,
            dlm: 0,
            isr: 0,
            ier: 0,
            fcr: 0,
            lcr: 0,
            mcr: 0,
            lsr: 0,
            msr: 0,
            scr: 0,
            val: 0,
        }
    }
}

impl BusDevice for Ns16550 {
    fn read(&mut self, addr: BusType) -> Result<BusType, BusError> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        match adj_addr as u64 {
            THR => {
                if self.lsr & LSR_DR != 0 {
                    self.lsr &= !LSR_DR;
                }
                Ok(0 as u32) // Todo: async(?) read
            }
            IER => return Ok(self.ier as BusType),
            ISR => return Ok(self.isr as BusType),
            LCR => return Ok(self.lcr as BusType),
            MCR => return Ok(self.mcr as BusType),
            LSR => return Ok(self.lsr as BusType),
            MSR => return Ok(self.msr as BusType),
            SCR => return Ok(self.scr as BusType),
            _ => Err(BusError::InvalidAddress),
        }
    }

    fn write(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), BusError> {
        let adj_addr = (addr as usize) - (self.get_begin_addr() as usize);

        match adj_addr as u64 {
            THR => {
                let c = data as u8 as char;
                print!("{}", c);
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
            _ => Err(BusError::InvalidAddress),
        }
    }

    fn get_begin_addr(&self) -> BusType {
        return UART_BASE_ADDRESS as BusType;
    }

    fn get_end_addr(&self) -> BusType {
        return (UART_BASE_ADDRESS + 8) as BusType;
    }

    fn tick(&mut self) {}
}
