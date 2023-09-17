mod util;
mod xmem;
use xmem::page_container::Xmem;
mod backend;
mod cpu;
mod frontend;

use crate::backend::ReturnableHandler;
use backend::ReturnableImpl;

fn fault() {
    println!("fault");
    ReturnableImpl::return_notify();
}

fn main() {
    let res = ReturnableImpl::handle(fault);

    println!("res: {:?}", res);
}
