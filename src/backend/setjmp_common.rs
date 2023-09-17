pub trait SetJmp {
    type JmpBuf;
    type SetJmpRet;
    type SigSetT;
    type SigJmpBuf;

    unsafe fn setjmp(env: &mut Self::JmpBuf) -> Self::SetJmpRet;
    unsafe fn longjmp(env: &mut Self::JmpBuf, val: Self::SetJmpRet) -> !;
    unsafe fn sigsetjmp(
        env: &mut Self::SigJmpBuf,
        mask: &Self::SigSetT,
        save_mask: bool,
    ) -> Self::SetJmpRet;
    unsafe fn siglongjmp(env: &mut Self::SigJmpBuf, val: Self::SetJmpRet) -> !;
}
