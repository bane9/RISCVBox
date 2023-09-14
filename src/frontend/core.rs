use crate::backend::common as JitCommon;
use crate::cpu;
use crate::xmem::page_container::Xmem;

use crate::frontend::csr;
use crate::frontend::privledged;
use crate::frontend::rva;
use crate::frontend::rvi;
use crate::frontend::rvm;

pub const CODE_PAGE_SIZE: usize = 4096;
pub const CODE_PAGE_READAHEAD: usize = 1;

pub struct Core {
    cpu: cpu::Cpu,
    xmem: Xmem,
    rom: Vec<u8>,
    offset: usize,
    total_ram_size: usize,
}

impl Core {
    pub fn new(rom: Vec<u8>, total_ram_size: usize) -> Core {
        assert!(rom.len() < total_ram_size);

        let mut xmem = Xmem::new_empty();
        let pages = (total_ram_size) / Xmem::page_size();
        xmem.realloc(pages).unwrap();

        let mut core = Core {
            cpu: cpu::Cpu::new(),
            xmem,
            rom,
            offset: 0,
            total_ram_size,
        };

        core.cpu.mem = core.xmem.as_ptr();

        core
    }

    pub fn parse(&mut self, start: usize, end: usize) -> Result<(), JitCommon::JitError> {
        let end = std::cmp::min(end, self.total_ram_size);
        assert!(start < end);
        let mut insn: u32 = 0;

        unsafe {
            std::ptr::copy_nonoverlapping(
                self.rom.as_ptr().add(start),
                &mut insn as *mut u32 as *mut u8,
                end - start,
            );
        }

        rvi::decode_rvi(&mut self.cpu, self.xmem.as_ptr(), insn)
    }
}
