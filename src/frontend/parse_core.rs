use crate::backend::common as JitCommon;
use crate::backend::common::BackendCore;
use crate::backend::common::HostEncodedInsn;

use crate::backend::target::core::BackendCoreImpl;
use crate::bus::bus;
use crate::cpu;
use crate::cpu::CpuReg;
use crate::cpu::Exception;
use crate::xmem::CodePage;

use crate::frontend::csr;
use crate::frontend::rva;
use crate::frontend::rvi;
use crate::frontend::rvm;
use crate::xmem::CodePageImpl;
use crate::xmem::PageState;

use super::code_pages::CodePages;

pub const INSN_SIZE: usize = 4; // Unlikely for rvc to be supported
pub const INSN_SIZE_BITS: usize = INSN_SIZE * 8;

pub const INSN_PAGE_SIZE: usize = 4096;
pub const INSN_PAGE_READAHEAD: usize = 1;

pub type DecoderFn = fn(u32) -> JitCommon::DecodeRet;

pub struct ParseCore {
    code_pages: CodePages,
}

impl ParseCore {
    pub fn new() -> ParseCore {
        ParseCore {
            code_pages: CodePages::new(),
        }
    }

    pub fn parse(
        &mut self,
        guest_start: usize,
        guest_end: usize,
    ) -> Result<(), JitCommon::JitError> {
        let mut insn: u32 = 0;

        let cpu = cpu::get_cpu();
        let bus = bus::get_bus();

        let code_page: &mut CodePageImpl;

        // lol
        unsafe {
            let self_mut = self as *mut Self;
            let (code_page_, _) = (*self_mut).code_pages.alloc_code_page();
            code_page = code_page_;
        }

        cpu.pc = guest_start as CpuReg;

        while (cpu.pc as usize) < guest_end {
            let loaded_insn = bus.fetch(cpu.pc, INSN_SIZE_BITS as u32);

            // if loaded_insn.is_err() {
            //     cpu.pc += INSN_SIZE as u32;
            //     continue;
            // }

            unsafe {
                std::ptr::copy_nonoverlapping(
                    &loaded_insn.unwrap() as *const u32 as *const u8,
                    &mut insn as *mut u32 as *mut u8,
                    INSN_SIZE,
                );
            }

            let result: Result<(), JitCommon::JitError>;

            unsafe {
                let code_page_mut = code_page as *mut CodePageImpl;
                result = self.decode_single(&mut *code_page_mut, insn);
            }

            cpu.pc += INSN_SIZE as u32;

            if let Err(JitCommon::JitError::ReachedBlockBoundary) = result {
                break;
            } else if result.is_err() {
                self.code_pages.mark_all_pages(PageState::ReadExecute);
                return result;
            }
        }

        code_page
            .push(BackendCoreImpl::emit_ret_with_exception(Exception::BlockExit).as_slice())
            .expect("Out of memory");

        code_page.mark_rx().unwrap();

        Ok(())
    }

    fn decode_single(
        &mut self,
        code_page: &mut CodePageImpl,
        insn: u32,
    ) -> Result<(), JitCommon::JitError> {
        static DECODERS: [DecoderFn; 4] = [
            rvi::decode_rvi,
            rvm::decode_rvm,
            rva::decode_rva,
            csr::decode_csr,
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

        let insn_res: HostEncodedInsn;

        match out_res {
            Ok(insn_res_unwrapped) => {
                insn_res = insn_res_unwrapped;
            }
            Err(JitCommon::JitError::InvalidInstruction(_)) => {
                insn_res =
                    BackendCoreImpl::emit_ret_with_exception(Exception::IllegalInstruction(insn));
            }
            _ => {
                return Err(out_res.err().unwrap());
            }
        }

        let host_insn_ptr = code_page.as_end_ptr();
        code_page.push(insn_res.as_slice()).expect("Out of memory");

        let cpu = cpu::get_cpu();

        cpu.insn_map.add_mapping(cpu.pc, host_insn_ptr);

        Ok(())
    }

    pub fn get_exec_ptr(&mut self, idx: usize) -> *mut u8 {
        self.code_pages.get_code_page(idx).as_ptr()
    }
}
