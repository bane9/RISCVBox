use crate::backend::common as JitCommon;
use crate::backend::common::BackendCore;
use crate::backend::returnable;
use crate::backend::target::core::BackendCoreImpl;
use crate::cpu;
use crate::frontend::code_pages;
use crate::xmem::page_container;
use crate::xmem::page_container::Xmem;

use crate::frontend::csr;
use crate::frontend::privledged;
use crate::frontend::rva;
use crate::frontend::rvi;
use crate::frontend::rvm;

use super::code_pages::CodePages;

pub const CODE_PAGE_SIZE: usize = 4096;
pub const CODE_PAGE_READAHEAD: usize = 1;

pub const INSN_SIZE: usize = 4; // Unlikely for rvc to be supported

pub type DecoderFn = fn(u32) -> JitCommon::DecodeRet;

pub struct ParseCore {
    code_pages: CodePages,
    rom: Vec<u8>,
    offset: usize,
    total_ram_size: usize,
}

extern "C" fn test_4_arg(arg1: usize, arg2: usize, arg3: usize, arg4: usize) -> usize {
    println!("test_4_arg: {:x} {:x} {:x} {:x}", arg1, arg2, arg3, arg4);
    0
}

impl ParseCore {
    pub fn new(rom: Vec<u8>) -> ParseCore {
        let pages = rom.len() / Xmem::page_size();
        let pages = pages + pages / 2;
        let pages = std::cmp::max(pages, 1);
        let mut code_pages = code_pages::CodePages::new(pages, CODE_PAGE_READAHEAD);

        BackendCoreImpl::fill_with_target_nop(code_pages.as_ptr(), pages * Xmem::page_size());

        let ok_jump = BackendCoreImpl::emit_ret_with_status(cpu::RunState::BlockExit);

        code_pages.apply_reserved_insn_all(ok_jump);

        code_pages.apply_insn(
            code_pages.as_ptr(),
            BackendCoreImpl::emit_usize_call_with_4_args(
                test_4_arg, 0xdead, 0xbeef, 0xcafe, 0xface,
            ),
        );

        code_pages.mark_all_pages(page_container::PageState::ReadExecute);

        let core = ParseCore {
            code_pages,
            rom,
            offset: 0,
            total_ram_size: pages * Xmem::page_size(),
        };

        core
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

    pub fn parse(&mut self, start: usize, end: usize) -> Result<(), JitCommon::JitError> {
        let end = std::cmp::min(end, self.total_ram_size);
        assert!(start < end);
        let mut insn: u32 = 0;

        let ptr = self.code_pages.as_ptr().wrapping_add(start);

        for i in (start..end).step_by(INSN_SIZE) {
            unsafe {
                std::ptr::copy_nonoverlapping(
                    self.rom.as_ptr().add(i),
                    &mut insn as *mut u32 as *mut u8,
                    INSN_SIZE,
                );
            }

            let result = self.decode_single(ptr.wrapping_add(i), insn, end);

            if let Err(JitCommon::JitError::ReachedBlockBoundary) = result {
                break;
            } else if result.is_err() {
                return result;
            }
        }

        self.code_pages
            .mark_all_pages(page_container::PageState::ReadExecute);

        Ok(())
    }

    fn decode_single(
        &mut self,
        ptr: *mut u8,
        insn: u32,
        block_boundary: usize,
    ) -> Result<(), JitCommon::JitError> {
        static DECODERS: [DecoderFn; 5] = [
            rvi::decode_rvi,
            rvm::decode_rvm,
            rva::decode_rva,
            csr::decode_csr,
            privledged::decode_privledged,
        ];

        let mut out_res: JitCommon::DecodeRet = Err(JitCommon::JitError::InvalidInstruction(insn));

        for decode in &DECODERS {
            let result = decode(insn);
            if let Err(JitCommon::JitError::InvalidInstruction(_)) = result {
                continue;
            } else {
                out_res = result;
                break;
            }
        }

        if out_res.is_err() {
            return Err(out_res.err().unwrap());
        }

        let out_res = out_res.unwrap();

        let result = self.code_pages.apply_insn(ptr, out_res);

        if result.is_none() {
            return Err(JitCommon::JitError::ReachedBlockBoundary);
        }

        let cpu = cpu::get_cpu();

        cpu.insn_map.insert(ptr, cpu.pc);

        cpu.pc += INSN_SIZE as u32;

        self.offset += out_res.size();

        Ok(())
    }

    pub fn get_exec_ptr(&self) -> *mut u8 {
        self.code_pages.as_ptr()
    }
}
