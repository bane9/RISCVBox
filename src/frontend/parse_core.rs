use crate::backend::common as JitCommon;
use crate::backend::common::BackendCore;
use crate::backend::common::HostEncodedInsn;

use crate::backend::target::core::BackendCoreImpl;
use crate::bus::bus;
use crate::bus::bus::BusType;
use crate::bus::mmu::AccessType;
use crate::cpu;
use crate::cpu::CpuReg;
use crate::cpu::Exception;
use crate::xmem::AllocationError;
use crate::xmem::CodePage;

use crate::frontend::csr;
use crate::frontend::rva;
use crate::frontend::rvi;
use crate::frontend::rvm;
use crate::xmem::PageState;

use super::code_pages::CodePages;

pub const INSN_SIZE: usize = 4; // Unlikely for rvc to be supported
pub const INSN_SIZE_BITS: usize = INSN_SIZE * 8;

pub const INSN_PAGE_SIZE: usize = 4096;
pub const INSN_PAGE_READAHEAD: usize = 1;

pub const RV_PAGE_SHIFT: usize = 12;
pub const RV_PAGE_SIZE: usize = 1 << RV_PAGE_SHIFT;
pub const RV_PAGE_OFFSET_MASK: usize = RV_PAGE_SIZE - 1;
pub const RV_PAGE_MASK: usize = !RV_PAGE_OFFSET_MASK;

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

    pub fn invalidate(&mut self, gpfn: CpuReg, should_reparse: bool) {
        let cpu = cpu::get_cpu();
        let bus = bus::get_bus();

        let phys_gpfn = bus
            .translate(gpfn << RV_PAGE_SHIFT, &mut cpu.mmu, AccessType::Fetch)
            .expect("Failed to translate gpfn for invalidation");

        cpu.gpfn_state.remove_gpfn(phys_gpfn);

        let idx: usize = cpu
            .insn_map
            .get_by_guest_idx(phys_gpfn)
            .unwrap()
            .jit_block_idx;

        self.code_pages.remove_code_page(idx);

        cpu.insn_map.remove_by_guest_page(phys_gpfn);

        if should_reparse {
            self.parse_gpfn(Some(gpfn))
                .expect("Failed to parse page after invalidation");
        } else {
            crate::xmem::PageAllocator::mark_page(phys_gpfn as *mut u8, 1, PageState::ReadWrite)
                .expect("Failed to mark guest page as readwrite after invalidation");
        }
    }

    pub fn parse_gpfn(&mut self, gpfn: Option<BusType>) -> Result<(), JitCommon::JitError> {
        let cpu = cpu::get_cpu();
        let bus = bus::get_bus();

        let gpfn = match gpfn {
            Some(gpfn) => gpfn,
            None => cpu.next_pc >> RV_PAGE_SHIFT as CpuReg,
        };

        assert!((gpfn as usize) << RV_PAGE_SHIFT < BusType::MAX as usize);

        let code_page: &mut CodePage;
        let code_page_idx: usize;

        unsafe {
            let self_mut = self as *mut Self;
            let (code_page_, code_page_idx_) = (*self_mut).code_pages.alloc_code_page();
            code_page = code_page_;
            code_page_idx = code_page_idx_;
        }

        let base_addr = bus
            .translate(gpfn << RV_PAGE_SHIFT, &mut cpu.mmu, AccessType::Load)
            .unwrap() as BusType;

        cpu.gpfn_state.add_gpfn(base_addr as CpuReg);

        cpu.current_gpfn = base_addr >> RV_PAGE_SHIFT as BusType;
        cpu.current_guest_page = base_addr;
        cpu.current_gpfn_offset = 0;

        while cpu.current_gpfn_offset < RV_PAGE_SIZE as BusType {
            let loaded_insn = bus.fetch_nommu(
                base_addr | cpu.current_gpfn_offset,
                INSN_SIZE_BITS as BusType,
            );

            let insn = match loaded_insn {
                Ok(insn) => insn,
                Err(_) => 0,
            };

            let result: Result<(), JitCommon::JitError>;

            unsafe {
                let code_page_mut = code_page as *mut CodePage;
                result = self.decode_single(
                    &mut *code_page_mut,
                    code_page_idx,
                    insn,
                    base_addr | cpu.current_gpfn_offset,
                );
            }

            cpu.current_gpfn_offset += INSN_SIZE as u32;

            if let Err(JitCommon::JitError::ReachedBlockBoundary) = result {
                break;
            } else if result.is_err() {
                cpu.current_gpfn = gpfn;
                cpu.current_guest_page = gpfn << RV_PAGE_SHIFT;
                self.code_pages.mark_all_pages(PageState::ReadExecute);
                return result;
            }
        }

        code_page
            .push(BackendCoreImpl::emit_ret_with_exception(Exception::BlockExit).as_slice())
            .expect("Out of memory");

        if cpu.insn_patch_list.len() > 0 {
            BackendCoreImpl::patch_jump_list(&cpu.insn_patch_list);
            cpu.insn_patch_list.clear();
        }

        code_page.mark_rx().unwrap();

        cpu.current_gpfn = gpfn;
        cpu.current_guest_page = gpfn << RV_PAGE_SHIFT;

        cpu.gpfn_state
            .set_gpfn_state(base_addr as CpuReg, PageState::ReadExecute);

        Ok(())
    }

    fn decode_single(
        &mut self,
        code_page: &mut CodePage,
        code_page_idx: usize,
        insn: u32,
        current_address: BusType,
    ) -> Result<(), JitCommon::JitError> {
        static DECODERS: [DecoderFn; 4] = [
            rvi::decode_rvi,
            rvm::decode_rvm,
            csr::decode_csr,
            rva::decode_rva,
        ];

        let mut out_res: JitCommon::DecodeRet = Err(JitCommon::JitError::InvalidInstruction(insn));

        let cpu = cpu::get_cpu();

        let host_insn_ptr = code_page.as_end_ptr();
        cpu.jit_current_ptr = host_insn_ptr;

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

        code_page.push(insn_res.as_slice()).expect("Out of memory");

        let cpu = cpu::get_cpu();

        cpu.insn_map
            .add_mapping(current_address, host_insn_ptr, code_page_idx);

        Ok(())
    }

    pub fn get_exec_ptr(&mut self, idx: usize) -> *mut u8 {
        self.code_pages.get_code_page(idx).as_ptr()
    }

    pub fn mark_page_state(&mut self, idx: usize, state: PageState) -> Result<(), AllocationError> {
        match state {
            PageState::ReadWrite => self.code_pages.get_code_page(idx).mark_rw(),
            PageState::ReadExecute => self.code_pages.get_code_page(idx).mark_rx(),
            PageState::Invalid => panic!("Invalid page state"),
        }
    }

    pub fn cleanup(&mut self) {
        self.code_pages.cleanup();
    }
}
