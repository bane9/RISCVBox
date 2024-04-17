use crate::bus::*;
use crate::cpu::*;

pub const PLIC_BASE: BusType = 0xc000000;
const PLIC_END: BusType = PLIC_BASE + 0x208000;

const SOURCE_PRIORITY: BusType = PLIC_BASE;
const SOURCE_PRIORITY_END: BusType = PLIC_BASE + 0xfff;

const PENDING: BusType = PLIC_BASE + 0x1000;
const PENDING_END: BusType = PLIC_BASE + 0x107f;

const ENABLE: BusType = PLIC_BASE + 0x2000;
const ENABLE_END: BusType = PLIC_BASE + 0x20ff;

const THRESHOLD_AND_CLAIM: BusType = PLIC_BASE + 0x200000;
const THRESHOLD_AND_CLAIM_END: BusType = PLIC_BASE + 0x201007;

const WORD_SIZE: BusType = 0x4;
const CONTEXT_OFFSET: BusType = 0x1000;
const SOURCE_NUM: BusType = 1024;

pub struct Plic {
    priority: [u32; SOURCE_NUM as usize],
    pending: [u32; 32],
    enable: [u32; 64],
    threshold: [u32; 2],
    claim: [u32; 2],
}

impl Plic {
    pub fn new() -> Plic {
        Self {
            priority: [0; 1024],
            pending: [0; 32],
            enable: [0; 64],
            threshold: [0; 2],
            claim: [0; 2],
        }
    }

    pub fn update_pending(&mut self, irq: u64) {
        let index = irq.wrapping_div(WORD_SIZE.into());
        self.pending[index as usize] = self.pending[index as usize] | (1 << irq);

        self.update_claim(irq);
    }

    fn clear_pending(&mut self, irq: u64) {
        let index = irq.wrapping_div(WORD_SIZE.into());
        self.pending[index as usize] = self.pending[index as usize] & !(1 << irq);

        self.update_claim(0);
    }

    fn update_claim(&mut self, irq: u64) {
        if self.is_enabled(1, irq) || irq == 0 {
            self.claim[1] = irq as u32;
        }
    }

    fn is_enabled(&self, context: u64, irq: u64) -> bool {
        let index = (irq.wrapping_rem(SOURCE_NUM.into())).wrapping_div((WORD_SIZE * 8).into());
        let offset = (irq.wrapping_rem(SOURCE_NUM.into())).wrapping_rem((WORD_SIZE * 8).into());
        return ((self.enable[(context * 32 + index) as usize] >> offset) & 1) == 1;
    }
}

impl BusDevice for Plic {
    fn load(&mut self, addr: BusType, size: BusType) -> Result<BusType, Exception> {
        if size != 32 {
            return Err(Exception::LoadAccessFault(addr));
        }

        match addr {
            SOURCE_PRIORITY..=SOURCE_PRIORITY_END => {
                if (addr - SOURCE_PRIORITY).wrapping_rem(WORD_SIZE) != 0 {
                    return Err(Exception::LoadAccessFault(addr));
                }
                let index = (addr - SOURCE_PRIORITY).wrapping_div(WORD_SIZE);
                Ok(self.priority[index as usize] as BusType)
            }
            PENDING..=PENDING_END => {
                if (addr - PENDING).wrapping_rem(WORD_SIZE) != 0 {
                    return Err(Exception::LoadAccessFault(addr));
                }
                let index = (addr - PENDING).wrapping_div(WORD_SIZE);
                Ok(self.pending[index as usize] as BusType)
            }
            ENABLE..=ENABLE_END => {
                if (addr - ENABLE).wrapping_rem(WORD_SIZE) != 0 {
                    return Err(Exception::LoadAccessFault(addr));
                }
                let index = (addr - ENABLE).wrapping_div(WORD_SIZE);
                Ok(self.enable[index as usize] as BusType)
            }
            THRESHOLD_AND_CLAIM..=THRESHOLD_AND_CLAIM_END => {
                let context = (addr - THRESHOLD_AND_CLAIM).wrapping_div(CONTEXT_OFFSET);
                let offset = addr - (THRESHOLD_AND_CLAIM + CONTEXT_OFFSET * context);
                if offset == 0 {
                    Ok(self.threshold[context as usize] as BusType)
                } else if offset == 4 {
                    Ok(self.claim[context as usize] as BusType)
                } else {
                    Err(Exception::LoadAccessFault(addr))
                }
            }
            _ => return Err(Exception::LoadAccessFault(addr)),
        }
    }

    fn store(&mut self, addr: BusType, data: BusType, size: BusType) -> Result<(), Exception> {
        if size != 32 {
            return Err(Exception::StoreAccessFault(addr));
        }

        match addr {
            SOURCE_PRIORITY..=SOURCE_PRIORITY_END => {
                if (addr - SOURCE_PRIORITY).wrapping_rem(WORD_SIZE) != 0 {
                    return Err(Exception::StoreAccessFault(addr));
                }
                let index = (addr - SOURCE_PRIORITY).wrapping_div(WORD_SIZE);
                self.priority[index as usize] = data as u32;
            }
            PENDING..=PENDING_END => {
                if (addr - PENDING).wrapping_rem(WORD_SIZE) != 0 {
                    return Err(Exception::StoreAccessFault(addr));
                }
                let index = (addr - PENDING).wrapping_div(WORD_SIZE);
                self.pending[index as usize] = data as u32;
            }
            ENABLE..=ENABLE_END => {
                if (addr - ENABLE).wrapping_rem(WORD_SIZE) != 0 {
                    return Err(Exception::StoreAccessFault(addr));
                }
                let index = (addr - ENABLE).wrapping_div(WORD_SIZE);
                self.enable[index as usize] = data as u32;
            }
            THRESHOLD_AND_CLAIM..=THRESHOLD_AND_CLAIM_END => {
                let context = (addr - THRESHOLD_AND_CLAIM).wrapping_div(CONTEXT_OFFSET);
                let offset = addr - (THRESHOLD_AND_CLAIM + CONTEXT_OFFSET * context);
                if offset == 0 {
                    self.threshold[context as usize] = data as u32;
                } else if offset == 4 {
                    self.clear_pending(data.into());
                } else {
                    Err(Exception::StoreAccessFault(addr))?
                }
            }
            _ => return Err(Exception::StoreAccessFault(addr)),
        }

        Ok(())
    }

    fn get_begin_addr(&self) -> BusType {
        PLIC_BASE as BusType
    }

    fn get_end_addr(&self) -> BusType {
        PLIC_END as BusType
    }

    fn tick_core_local(&mut self) {}

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_from_main_thread(&mut self) {}

    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        None
    }
}
