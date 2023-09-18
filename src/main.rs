mod util;
mod xmem;
use xmem::page_container::Xmem;
mod backend;
mod cpu;
mod frontend;

use crate::backend::ReturnableHandler;
use backend::ReturnableImpl;

fn main() {
    let mut core = frontend::parse_core::Core::new(Vec::new(), 4096);
}
