#![allow(dead_code)]
#![feature(thread_local)]
#[allow(arithmetic_overflow)]
mod backend;
mod bus;
mod cpu;
mod frontend;
mod util;
mod window;
mod xmem;

use backend::csr::init_backend_csr;
use bus::ram::RAM_BEGIN_ADDR;
use frontend::exec_core::ExecCoreThreadPool;

use crate::bus::BusDevice;

fn init_bus(mut rom: Vec<u8>, ram_size: usize, dtb: Option<Vec<u8>>) {
    assert!(ram_size >= rom.len());
    // There are roughly sorted by expected frequency of access

    let bus = bus::bus::get_bus();

    rom.resize(ram_size, 0);

    let mut ram = bus::ram::Ram::new(rom);
    let ram_ptr = ram.get_ptr(RAM_BEGIN_ADDR).unwrap();

    bus.set_ram_ptr(ram_ptr, ram.get_end_addr() as usize);

    bus.add_device(Box::new(ram));

    let ns16550 = bus::ns16550::Ns16550::new();

    bus.add_device(Box::new(ns16550));

    let plic = bus::plic::Plic::new();

    bus.add_device(Box::new(plic));

    let clint = bus::clint::Clint::new();

    bus.add_device(Box::new(clint));

    if let Some(dtb) = dtb {
        let dtb = bus::dtb::Dtb::new(&dtb);

        bus.add_device(Box::new(dtb));
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

    let mut rom = util::read_file("buildroot/images1/fw_jump.bin").unwrap();

    rom.resize(util::size_mib(4), 0);
    let mut kernel = util::read_file("buildroot/ImageRFS").unwrap();
    rom.append(&mut kernel);

    let dtb = Some(util::read_file("buildroot/images/dtb.dtb").unwrap());

    init_backend_csr();

    window::ConsoleSettings::set_interactive_console();

    init_bus(rom, ram_size, dtb);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    exec_thread_pool.join();
}
