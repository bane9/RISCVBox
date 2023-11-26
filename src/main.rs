#![allow(dead_code)]

mod backend;
mod bus;
mod cpu;
mod frontend;
mod util;
mod xmem;

use backend::csr::init_backend_csr;
use bus::ram::RAM_BEGIN_ADDR;
use frontend::exec_core::ExecCoreThreadPool;

fn init_bus(mut rom: Vec<u8>, ram_size: usize, dtb: Option<Vec<u8>>) {
    assert!(ram_size >= rom.len());

    rom.resize(ram_size, 0);

    let ram = bus::ram::Ram::new(rom);

    bus::bus::get_bus().add_device(Box::new(ram));

    let clint = bus::clint::Clint::new();

    bus::bus::get_bus().add_device(Box::new(clint));

    let ns16550 = bus::ns16550::Ns16550::new();

    bus::bus::get_bus().add_device(Box::new(ns16550));

    let plic = bus::plic::Plic::new();

    bus::bus::get_bus().add_device(Box::new(plic));

    if let Some(dtb) = dtb {
        let dtb = bus::dtb::Dtb::new(&dtb);

        bus::bus::get_bus().add_device(Box::new(dtb));
    }
}

fn main() {
    let ram_size = util::size_mib(128);

    // let argv = std::env::args().collect::<Vec<String>>();

    // if argv.len() < 2 {
    //     println!("Usage: {} <bin> [timeout]", argv[0]);
    //     std::process::exit(1);
    // }

    // let rom = util::read_file(&argv[1]).unwrap();

    // let dtb = if argv.len() == 3 {
    //     Some(util::read_file(&argv[2]).unwrap())
    // } else {
    //     None
    // };

    let rom = util::read_file("buildroot/images/linux.bin").unwrap();
    let dtb = Some(util::read_file("buildroot/images/dtb.dtb").unwrap());

    init_backend_csr();

    init_bus(rom.clone(), ram_size, dtb);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    exec_thread_pool.join();
}
