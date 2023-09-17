mod util;
mod xmem;
use xmem::page_container::Xmem;
mod backend;
mod cpu;
mod frontend;

use backend::setjmp::*;

declare_setjmp_buffer!(JMP_BUF);

// #[inline(never)]
// fn jmp_fn() {
//     println!("jmp_fn");
//     call_longjmp!(JMP_BUF, 1);

//     panic!("unreachable");
// }

// fn main() {
//     if call_setjmp!(JMP_BUF) == 1 {
//         println!("longjmp");

//         return;
//     }

//     jmp_fn();
// }

pub unsafe fn test_longjmp(ptr: *mut c_void) {
    longjmp(ptr, 1);
}

fn main() {
    pub static mut ARR: [u8; 256] = [0; 256];

    if unsafe { setjmp(ARR.as_mut_ptr() as _) } == 0 {
        println!("setjmp");
        unsafe { test_longjmp(ARR.as_mut_ptr() as _) };
    } else {
        println!("longjmp");
    }
}
