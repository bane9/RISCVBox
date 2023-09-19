mod backend;
mod bus;
mod cpu;
mod frontend;
mod util;
mod xmem;

use frontend::exec_core::ExecCoreThreadPool;

fn init_bus(mut rom: Vec<u8>, ram_size: usize) {
    assert!(ram_size >= rom.len());

    rom.resize(ram_size, 0);

    let ram = bus::ram::Ram::new(rom);

    bus::bus::get_bus().add_device(Box::new(ram));
}

fn main() {
    let ram_size = 4096;
    let rom = util::read_file("test.bin").unwrap();

    init_bus(rom.clone(), ram_size);

    let exec_thread_pool = ExecCoreThreadPool::new(rom, 1);

    exec_thread_pool.join();
}
