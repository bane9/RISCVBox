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

use std::io::Write;

use backend::csr::init_backend_csr;
use bus::{ram::RAM_BEGIN_ADDR, ramfb::RAMFB_BEGIN_ADDR};
use frontend::exec_core::ExecCoreThreadPool;

use crate::bus::BusDevice;

use vm_fdt::FdtWriter;

fn create_dtb(
    ram_origin: u32,
    ram_size: u32,
    width: usize,
    height: usize,
    bpp: usize,
    using_fb: bool,
) -> Vec<u8> {
    //TODO: make devices describe themselves
    let mut fdt = FdtWriter::new().unwrap();

    let root_node = fdt.begin_node("").unwrap();
    fdt.property_u32("#address-cells", 0x2).unwrap();
    fdt.property_u32("#size-cells", 0x2).unwrap();
    fdt.property_string("compatible", "riscv-virtio").unwrap();
    fdt.property_string("model", "riscv-virtio,qemu").unwrap();

    let chosen_node = fdt.begin_node("chosen").unwrap();
    fdt.property_string(
        "bootargs",
        "earlycon=sbi random.trust_bootloader=on fbcon=nodefer fbcon=map:0 console=ttyS0",
    )
    .unwrap();
    fdt.property_u32("rng-seed", 0x4).unwrap();
    fdt.end_node(chosen_node).unwrap();

    let fdt_memory_node = fdt.begin_node("memory@80000000").unwrap();
    fdt.property_string("device_type", "memory").unwrap();
    fdt.property_array_u32("reg", &[0x00, ram_origin as u32, 0x00, ram_size as u32])
        .unwrap();
    fdt.end_node(fdt_memory_node).unwrap();

    let cpus_node = fdt.begin_node("cpus").unwrap();
    fdt.property_u32("#address-cells", 0x1).unwrap();
    fdt.property_u32("#size-cells", 0x0).unwrap();
    fdt.property_u32("timebase-frequency", 100000000).unwrap();

    let cpu_node = fdt.begin_node("cpu@0").unwrap();
    fdt.property_u32("phandle", 0x1).unwrap();
    fdt.property_string("device_type", "cpu").unwrap();
    fdt.property_u32("reg", 0x0).unwrap();
    fdt.property_string("status", "okay").unwrap();
    fdt.property_string("compatible", "riscv").unwrap();
    fdt.property_string("riscv,isa", "rv32imasu").unwrap();
    fdt.property_string("mmu-type", "riscv,sv32").unwrap();

    // Begin interrupt controller node
    let cpu0_intc_phandle = 0x02u32;

    let cpu0_intc_node = fdt.begin_node("cpu0_intc").unwrap();
    fdt.property_u32("#interrupt-cells", 0x01).unwrap();
    fdt.property_u32("#address-cells", 0x00).unwrap();
    fdt.property_null("interrupt-controller").unwrap();
    fdt.property_string("compatible", "riscv,cpu-intc").unwrap();
    fdt.property_u32("phandle", cpu0_intc_phandle).unwrap();
    fdt.end_node(cpu0_intc_node).unwrap();
    // End interrupt controller node

    fdt.end_node(cpu_node).unwrap();
    fdt.end_node(cpus_node).unwrap();

    // Begin cpu-map node
    let cpu_map_node = fdt.begin_node("cpu-map").unwrap();
    let cluster0_node = fdt.begin_node("cluster0").unwrap();
    let core0_node = fdt.begin_node("core0").unwrap();
    fdt.property_u32("cpu", 0x01).unwrap();
    fdt.end_node(core0_node).unwrap();
    fdt.end_node(cluster0_node).unwrap();
    fdt.end_node(cpu_map_node).unwrap();
    // End cpu-map node

    let soc_node = fdt.begin_node("soc").unwrap();
    fdt.property_u32("#address-cells", 0x2).unwrap();
    fdt.property_u32("#size-cells", 0x2).unwrap();
    fdt.property_string("compatible", "simple-bus").unwrap();
    fdt.property_null("ranges").unwrap();

    // Begin plic node
    let plic_phandle = 0x03u32;

    let plic_node = fdt.begin_node("plic@c000000").unwrap();
    fdt.property_u32("phandle", plic_phandle).unwrap();
    fdt.property_u32("riscv,ndev", 0x60).unwrap();
    fdt.property_array_u32("reg", &[0x00, 0xc000000, 0x00, 0x600000])
        .unwrap();
    fdt.property_array_u32(
        "interrupts-extended",
        &[cpu0_intc_phandle, 0x0b, cpu0_intc_phandle, 0x09],
    )
    .unwrap();
    fdt.property_string_list(
        "compatible",
        vec!["sifive,plic-1.0.0".into(), "riscv,plic0".into()],
    )
    .unwrap();
    fdt.property_null("interrupt-controller").unwrap();
    fdt.property_u32("#interrupt-cells", 0x01).unwrap();
    fdt.property_u32("#address-cells", 0x00).unwrap();
    fdt.end_node(plic_node).unwrap();
    // End plic node

    // Begin serial node
    let serial_node = fdt.begin_node("serial@10000000").unwrap();
    fdt.property_u32("interrupts", 0x0a).unwrap();
    fdt.property_u32("interrupt-parent", plic_phandle).unwrap();
    fdt.property_string("clock-frequency", "115200").unwrap();
    fdt.property_array_u32("reg", &[0x00, 0x10000000, 0x00, 0x8])
        .unwrap();
    fdt.property_string("compatible", "ns16550a").unwrap();
    fdt.end_node(serial_node).unwrap();
    // End serial node

    // Begin clint node
    let clint_node = fdt.begin_node("clint@2000000").unwrap();
    fdt.property_array_u32(
        "interrupts-extended",
        &[cpu0_intc_phandle, plic_phandle, cpu0_intc_phandle, 0x07],
    )
    .unwrap();
    fdt.property_array_u32("reg", &[0x00, 0x2000000, 0x00, 0x10000])
        .unwrap();
    fdt.property_string_list(
        "compatible",
        vec!["sifive,clint0".into(), "riscv,clint0".into()],
    )
    .unwrap();
    fdt.end_node(clint_node).unwrap();
    // End clint node

    // Begin framebuffer node
    if using_fb {
        let bytes_per_pixel = bpp / 8;

        let framebuffer_node = fdt.begin_node("framebuffer@1d380000").unwrap();
        fdt.property_string("compatible", "simple-framebuffer")
            .unwrap();
        fdt.property_array_u32(
            "reg",
            &[
                0x00,
                0x1d380000,
                0x00,
                (width * height * bytes_per_pixel) as u32,
            ],
        )
        .unwrap();
        fdt.property_u32("width", width as u32).unwrap();
        fdt.property_u32("height", height as u32).unwrap();
        fdt.property_u32("stride", (width * bytes_per_pixel) as u32)
            .unwrap();
        fdt.property_string("format", "a8b8g8r8").unwrap();

        fdt.end_node(framebuffer_node).unwrap();
    }
    // End framebuffer node

    fdt.end_node(soc_node).unwrap();

    fdt.end_node(root_node).unwrap();

    let res = fdt.finish().unwrap();

    // dump the result to a file
    let mut f = std::fs::File::create("buildroot/images/dtb_dump.dtb").unwrap();
    f.write_all(&res).unwrap();

    res
}

fn init_bus(
    mut rom: Vec<u8>,
    ram_size: usize,
    width: usize,
    height: usize,
    bpp: usize,
    using_fb: bool,
) {
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

    let mut ramfb = bus::ramfb::RamFB::new(width, height, bpp);
    let fb_ptr = ramfb.get_fb_ptr();

    bus.set_fb_ptr(fb_ptr, ramfb.get_end_addr() as usize);

    bus.add_device(Box::new(ramfb));

    let dtb = create_dtb(
        RAM_BEGIN_ADDR,
        ram_size as u32,
        width,
        height,
        bpp,
        using_fb,
    );

    let dtb = bus::dtb::Dtb::new(&dtb);

    bus.add_device(Box::new(dtb));
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
    let mut kernel = util::read_file("buildroot/ImageRFS1").unwrap();
    rom.append(&mut kernel);

    init_backend_csr();

    window::ConsoleSettings::set_interactive_console();

    let width = 800;
    let height = 600;
    let bpp = 32;
    let using_fb = false;

    init_bus(rom, ram_size, width, height, bpp, using_fb);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    if using_fb {
        let mut window = window::window::Window::new(RAMFB_BEGIN_ADDR as *mut u8, width, height);

        window.event_loop();
    }

    exec_thread_pool.join();
}
