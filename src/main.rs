mod util;
mod xmem;
use xmem::page_container::Xmem;
mod backend;
mod cpu;
mod frontend;

fn main() {
    let file = "test.bin";
    let rom = util::read_file(file).unwrap();

    let mut parse_core = frontend::parse_core::Core::new(rom, 4096 * 1024);

    parse_core.parse_ahead().unwrap();

    println!("done");
}
