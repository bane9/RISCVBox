#![allow(dead_code)]

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
use window::WindowCommon;

use crate::bus::ps2keyboard;

struct InitData {
    fb_ptr: *mut u8,
}

fn init_bus(
    mut rom: Vec<u8>,
    ram_size: usize,
    dtb: Option<Vec<u8>>,
    fb_width: usize,
    fb_height: usize,
    fb_bpp: usize,
) -> InitData {
    assert!(ram_size >= rom.len());
    // There are roughly sorted by expected frequency of access

    rom.resize(ram_size, 0);

    let ram = bus::ram::Ram::new(rom);

    bus::bus::get_bus().add_device(Box::new(ram));

    let mut ramfb = bus::ramfb::RamFB::new(fb_width, fb_height, fb_bpp);
    let fb_ptr = ramfb.get_fb_ptr();

    bus::bus::get_bus().add_device(Box::new(ramfb));

    let clint = bus::clint::Clint::new();

    bus::bus::get_bus().add_device(Box::new(clint));

    let ns16550 = bus::ns16550::Ns16550::new();

    bus::bus::get_bus().add_device(Box::new(ns16550));

    let plic = bus::plic::Plic::new();

    bus::bus::get_bus().add_device(Box::new(plic));

    let ps2keyboard = ps2keyboard::PS2Keyboard::new();

    bus::bus::get_bus().add_device(Box::new(ps2keyboard));

    if let Some(dtb) = dtb {
        let dtb = bus::dtb::Dtb::new(&dtb);

        bus::bus::get_bus().add_device(Box::new(dtb));
    }

    InitData { fb_ptr }
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

    let rom = util::read_file("buildroot/doomgeneric.bin").unwrap();
    let dtb = Some(util::read_file("buildroot/images/dtb.dtb").unwrap());

    init_backend_csr();

    let width = 800;
    let height = 600;
    let bpp = 32;

    let init_data = init_bus(rom.clone(), ram_size, dtb, width, height, bpp);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    let mut window =
        window::window_impl::new(width, height, bpp, "RISCVBox", init_data.fb_ptr, false);

    window.event_loop();

    exec_thread_pool.join();
}
