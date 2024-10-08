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

use clap::Parser;

use backend::csr::init_backend_csr;
use bus::{
    ram::RAM_BEGIN_ADDR,
    ramfb::RAMFB_BEGIN_ADDR,
    syscon::{SYSCON_ADDR, SYSCON_POWEROFF, SYSCON_REBOOT, SYSCON_SIZE},
    tlb::{self, asid_tlb_init},
};
use cpu::{csr, CPU_INTC_PHANDLE, CPU_TIMEBASE_FREQ};
use frontend::exec_core::ExecCoreThreadPool;

use crate::bus::BusDevice;

use vm_fdt::FdtWriter;

fn create_dtb(ram_origin: u32, ram_size: u32, has_fb: bool) -> Vec<u8> {
    let mut fdt: FdtWriter = FdtWriter::new().unwrap();

    let root_node = fdt.begin_node("").unwrap();
    fdt.property_u32("#address-cells", 0x2).unwrap();
    fdt.property_u32("#size-cells", 0x2).unwrap();
    fdt.property_string("compatible", "riscv-virtio").unwrap();
    fdt.property_string("model", "riscv-virtio,qemu").unwrap();

    let chosen_node = fdt.begin_node("chosen").unwrap();

    let bootargs = if has_fb {
        "fbcon=nodefer fbcon=map:0"
    } else {
        "earlycon=sbi console=ttyS0"
    };

    fdt.property_string("bootargs", bootargs).unwrap();

    fdt.end_node(chosen_node).unwrap();

    let fdt_memory_node = fdt
        .begin_node(&util::fdt_node_addr_helper("memory", ram_origin))
        .unwrap();
    fdt.property_string("device_type", "memory").unwrap();
    fdt.property_array_u32("reg", &[0x00, ram_origin as u32, 0x00, ram_size as u32])
        .unwrap();
    fdt.end_node(fdt_memory_node).unwrap();

    let cpus_node = fdt.begin_node("cpus").unwrap();
    fdt.property_u32("#address-cells", 0x1).unwrap();
    fdt.property_u32("#size-cells", 0x0).unwrap();
    fdt.property_u32("timebase-frequency", CPU_TIMEBASE_FREQ)
        .unwrap();

    let cpu_node = fdt.begin_node("cpu@0").unwrap();
    fdt.property_u32("phandle", 0x1).unwrap();
    fdt.property_string("device_type", "cpu").unwrap();
    fdt.property_u32("reg", 0x0).unwrap();
    fdt.property_string("status", "okay").unwrap();
    fdt.property_string("compatible", "riscv").unwrap();
    fdt.property_string("riscv,isa", "rv32imasu").unwrap();
    fdt.property_string("mmu-type", "riscv,sv32").unwrap();

    // Begin syscon node
    let syscon_regmap = &[0x00, SYSCON_ADDR, 0x00, SYSCON_SIZE];
    let syscnon_node = fdt
        .begin_node(&util::fdt_node_addr_helper("syscon", SYSCON_ADDR))
        .unwrap();
    fdt.property_u32("phandle", 0x4).unwrap();
    fdt.property_array_u32("reg", syscon_regmap).unwrap();
    fdt.property_string_list(
        "compatible",
        vec![
            "sifive,test1".into(),
            "sifive,test0".into(),
            "syscon".into(),
        ],
    )
    .unwrap();
    fdt.end_node(syscnon_node).unwrap();

    let poweroff_node = fdt.begin_node("poweroff").unwrap();
    fdt.property_string("compatible", "syscon-poweroff")
        .unwrap();
    fdt.property_u32("value", SYSCON_POWEROFF).unwrap();
    fdt.property_u32("offset", 0).unwrap();
    fdt.property_array_u32("regmap", syscon_regmap).unwrap();
    fdt.end_node(poweroff_node).unwrap();

    let reboot_node = fdt.begin_node("reboot").unwrap();
    fdt.property_string("compatible", "syscon-reboot").unwrap();
    fdt.property_u32("value", SYSCON_REBOOT).unwrap();
    fdt.property_array_u32("regmap", syscon_regmap).unwrap();
    fdt.property_u32("offset", 0).unwrap();
    fdt.end_node(reboot_node).unwrap();
    // End syscon node

    // Begin interrupt controller node
    let cpu0_intc_node = fdt.begin_node("cpu0_intc").unwrap();
    fdt.property_u32("#interrupt-cells", 0x01).unwrap();
    fdt.property_u32("#address-cells", 0x00).unwrap();
    fdt.property_null("interrupt-controller").unwrap();
    fdt.property_string("compatible", "riscv,cpu-intc").unwrap();
    fdt.property_u32("phandle", CPU_INTC_PHANDLE).unwrap();
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

    bus::get_bus().describe_fdts(&mut fdt);

    fdt.end_node(soc_node).unwrap();

    fdt.end_node(root_node).unwrap();

    fdt.finish().unwrap()
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

    let mut ramfb = bus::ramfb::RamFB::new(width, height, bpp, using_fb);
    let fb_ptr = ramfb.get_fb_ptr();

    bus.set_fb_ptr(fb_ptr, ramfb.get_end_addr() as usize);

    bus.add_device(Box::new(ramfb));

    let clint = bus::clint::Clint::new();

    bus.add_device(Box::new(clint));

    let dtb = create_dtb(RAM_BEGIN_ADDR, ram_size as u32, using_fb);

    let dtb = bus::dtb::Dtb::new(&dtb);

    bus.add_device(Box::new(dtb));

    let syscon = bus::syscon::Syscon::new();

    bus.add_device(Box::new(syscon));
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long, help = "Path to BIOS (firmware) image")]
    bios: String,

    #[arg(short, long, default_value = "", help = "Path to Linux kernel image")]
    kernel: String,

    #[arg(short, long, default_value_t = 64, help = "Memory size in MiB")]
    memory: usize,

    #[arg(
        long,
        default_value_t = false,
        help = "Disable the graphical output (only output to console)"
    )]
    nographic: bool,

    #[arg(
        long,
        default_value_t = 800,
        help = "Width of the graphical output in pixels"
    )]
    width: usize,

    #[arg(
        long,
        default_value_t = 600,
        help = "Height of the graphical output in pixels"
    )]
    height: usize,

    #[arg(
        short,
        long,
        default_value_t = 1,
        help = "Scale factor for the graphical output (1, 2, 4, 8, 16, 32)"
    )]
    scale: usize,
}

fn run_emulator(args: &Args) {
    let rom = util::read_file(&args.bios);

    if rom.is_err() {
        println!("Failed to read bios file: {}", args.bios);
        std::process::exit(1);
    }

    let mut rom = rom.unwrap();

    if !args.kernel.is_empty() {
        let kernel = util::read_file(&args.kernel);

        if kernel.is_err() {
            println!("Failed to read kernel file: {}", args.kernel);
            std::process::exit(1);
        }

        let mut kernel = kernel.unwrap();

        rom.resize(util::size_mib(4), 0);
        rom.append(&mut kernel);
    }

    let ram_size = util::size_mib(args.memory);
    let using_fb = !args.nographic;

    util::init();
    init_backend_csr();
    asid_tlb_init();

    window::ConsoleSettings::set_interactive_console();

    let width = args.width;
    let height = args.height;
    let bpp = 32;

    init_bus(rom, ram_size, width, height, bpp, using_fb);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    if using_fb {
        let mut window =
            window::window::Window::new(RAMFB_BEGIN_ADDR as *mut u8, width, height, args.scale);

        window.event_loop();
    }

    exec_thread_pool.join();

    bus::cleanup();
    csr::cleanup_csr();
    tlb::cleanup_asid_tlb();
    cpu::remove_all_cpus();
}

fn main() {
    let args = Args::parse();

    loop {
        run_emulator(&args);

        if bus::syscon::should_reboot() {
            bus::syscon::clear_should_reboot();
        } else {
            return;
        }
    }
}
