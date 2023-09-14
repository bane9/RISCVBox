mod util;
mod xmem;
use xmem::page_container::Xmem;
mod backend;
mod cpu;
mod frontend;

fn main() {
    let file = "test.bin";
    let rom = util::read_file(file).unwrap();

    let mut core = frontend::core::Core::new(rom, 4096 * 1024);

    core.parse(0, 4).unwrap();

    println!("done");
}
