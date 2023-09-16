use crate::backend::common as JitCommon;
use crate::backend::common::BackendCore;
use crate::backend::target::core::BackendCoreImpl;
use crate::cpu;
use crate::xmem::page_container::Xmem;

use crate::frontend::csr;
use crate::frontend::privledged;
use crate::frontend::rva;
use crate::frontend::rvi;
use crate::frontend::rvm;

pub const CODE_PAGE_SIZE: usize = 4096;
pub const CODE_PAGE_READAHEAD: usize = 1;

pub const INSN_SIZE: usize = 4; // Unlikely for rvc to be supported

pub type DecoderFn = fn(&mut cpu::Cpu, *mut u8, u32) -> Result<(), JitCommon::JitError>;

pub struct Core {
    cpu: cpu::Cpu,
    xmem: Xmem,
    ram: Vec<u8>,
    offset: usize,
    total_ram_size: usize,
}

enum PageState {
    Invalid,
    RW,
    RX,
}

impl Core {
    pub fn new(rom: Vec<u8>, total_ram_size: usize) -> Core {
        assert!(rom.len() < total_ram_size);

        let mut xmem = Xmem::new_empty();
        let pages = rom.len() / Xmem::page_size();
        let pages = pages + pages / 2;
        xmem.realloc(pages).unwrap();

        BackendCoreImpl::fill_with_target_exc(xmem.as_ptr(), pages * Xmem::page_size());

        let mut ram = rom.clone();

        ram.resize(total_ram_size, 0);

        let cpu = cpu::Cpu::new(ram.as_ptr() as *mut u8);

        let core = Core {
            cpu,
            xmem,
            ram,
            offset: 0,
            total_ram_size,
        };

        core.mark_page_range(0, pages, PageState::Invalid);

        core
    }

    fn mark_page_range(&self, start: usize, end: usize, state: PageState) {
        match state {
            PageState::Invalid => {
                for i in start..end {
                    Xmem::mark_invalid(self.xmem.as_ptr().wrapping_add(i * Xmem::page_size()))
                        .expect_err("Failed to mark xmem invalid");
                }
            }
            PageState::RW => {
                for i in start..end {
                    Xmem::mark_rw(self.xmem.as_ptr().wrapping_add(i * Xmem::page_size()))
                        .expect_err("Failed to mark xmem rw");
                }
            }
            PageState::RX => {
                for i in start..end {
                    Xmem::mark_rx(self.xmem.as_ptr().wrapping_add(i * Xmem::page_size()))
                        .expect_err("Failed to mark xmem rx");
                }
            }
        }
    }

    pub fn parse(&mut self, start: usize, end: usize) -> Result<(), JitCommon::JitError> {
        let end = std::cmp::min(end, self.total_ram_size);
        assert!(start < end);
        let mut insn: u32 = 0;

        self.mark_page_range(
            start % Xmem::page_size(),
            end % Xmem::page_size(),
            PageState::RW,
        );

        let mut ptr = self.xmem.as_ptr().wrapping_add(start);

        for i in (start..end).step_by(INSN_SIZE) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.ram.as_ptr().add(i),
                    &mut insn as *mut u32 as *mut u8,
                    INSN_SIZE,
                );
            }

            match self.exec(ptr, insn) {
                Ok(_) => ptr = ptr.wrapping_add(INSN_SIZE),
                Err(e) => {
                    self.mark_page_range(
                        start % Xmem::page_size(),
                        end % Xmem::page_size(),
                        PageState::RX,
                    );
                    return Err(e);
                }
            }
        }

        self.mark_page_range(
            start % Xmem::page_size(),
            end % Xmem::page_size(),
            PageState::RX,
        );

        Ok(())
    }

    pub fn parse_ahead(&mut self) -> Result<(), JitCommon::JitError> {
        let start = self.offset;

        if start >= self.total_ram_size {
            return Ok(());
        }

        let end = std::cmp::min(
            start + CODE_PAGE_SIZE * CODE_PAGE_READAHEAD,
            self.total_ram_size,
        );

        self.parse(start, end)?;

        self.offset = end;

        Ok(())
    }

    // Make sure ptr is at rw page
    pub fn exec(&mut self, ptr: *mut u8, insn: u32) -> Result<(), JitCommon::JitError> {
        static DECODERS: [DecoderFn; 5] = [
            rvi::decode_rvi,
            rvm::decode_rvm,
            rva::decode_rva,
            csr::decode_csr,
            privledged::decode_privledged,
        ];

        for decode in &DECODERS {
            let result = decode(&mut self.cpu, ptr, insn);
            if let Err(JitCommon::JitError::InvalidInstruction(_)) = result {
                continue;
            } else {
                return result;
            }
        }

        Err(JitCommon::JitError::InvalidInstruction(insn))
    }
}
