use crate::backend::setjmp_common;
use setjmp;
use std::mem::MaybeUninit;

pub struct SetJmpImpl;

impl setjmp_common::SetJmp for SetJmpImpl {
    type JmpBuf = setjmp::jmp_buf;
    type SetJmpRet = i32;

    type SigSetT = i32;

    type SigJmpBuf = JmpBuf;

    unsafe fn setjmp(env: &mut jmp_buf) -> SetJmpRet {
        setjmp::setjmp(*env)
    }

    unsafe fn longjmp(env: &mut jmp_buf, val: SetJmpRet) -> ! {
        setjmp::longjmp(*env, val)
    }

    unsafe fn sigsetjmp(
        env: &mut Self::SigJmpBuf,
        mask: &Self::SigSetT,
        save_mask: bool,
    ) -> Self::SetJmpRet {
        todo!()
    }

    unsafe fn siglongjmp(env: &mut Self::SigJmpBuf, val: Self::SetJmpRet) -> ! {
        todo!()
    }
}
