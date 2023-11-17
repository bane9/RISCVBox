#![allow(dead_code)]

mod backend;
mod bus;
mod cpu;
mod frontend;
mod util;
mod xmem;

use std::thread;
use std::time::Duration;

use std::process::Output;

use bus::{ram::RAM_BEGIN_ADDR, BusType};
use cpu::Exception;
use frontend::exec_core::ExecCoreThreadPool;

const TOHOSTADDR: BusType = 0x01000000;

struct ToHost;

impl bus::BusDevice for ToHost {
    fn read(&mut self, _addr: BusType, _size: BusType) -> Result<BusType, Exception> {
        Ok(0)
    }

    fn write(&mut self, _addr: BusType, _data: BusType, _size: BusType) -> Result<(), Exception> {
        let cpu = cpu::get_cpu();

        let a0 = if cpu.regs[cpu::RegName::A0 as usize] == 0 {
            0
        } else {
            cpu.regs[cpu::RegName::A0 as usize] >> 1
        };
        let gp = cpu.regs[cpu::RegName::Gp as usize];

        println!("tohost a0: {}, gp {}", a0, gp);

        std::process::exit(if a0 == 0 && gp == 1 { 0 } else { 1 });
    }

    fn get_begin_addr(&self) -> BusType {
        TOHOSTADDR
    }

    fn get_end_addr(&self) -> BusType {
        TOHOSTADDR + 4
    }

    fn tick(&mut self) {}
}

fn init_bus(mut rom: Vec<u8>, ram_size: usize) {
    assert!(ram_size >= rom.len());

    rom.resize(ram_size, 0);

    let ram = bus::ram::Ram::new(rom);
    let tohost = ToHost {};

    bus::bus::get_bus().add_device(Box::new(ram));
    bus::bus::get_bus().add_device(Box::new(tohost));
}

fn timeout_thread() {
    thread::spawn(|| {
        let duration = Duration::from_secs(5);

        thread::sleep(duration);

        println!("Test timeout after {:?}", duration);

        std::process::exit(1);
    });
}

fn main() {
    let ram_size = util::size_kib(64);

    let argv = std::env::args().collect::<Vec<String>>();

    if argv.len() < 2 {
        println!("Usage: {} <bin> [timeout]", argv[0]);
        std::process::exit(1);
    }

    let rom = util::read_file(&argv[1]).unwrap();

    if argv.len() == 3 && argv[2] == "timeout" {
        timeout_thread();
    }

    // let arg = "testbins/rv32ui/bin/fence_i.bin";
    // let rom = util::read_file(arg).unwrap();

    init_bus(rom.clone(), ram_size);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    exec_thread_pool.join();

    std::process::exit(1); // It's only valid to exit from the tohost device
}

fn run_bin_as_subproccess(bin: &str) -> Output {
    let child = std::process::Command::new("target/debug/deps/test_riscv_isa")
        .arg(bin)
        .arg("timeout")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output()
        .unwrap();

    child
}

fn list_files_from_directory(dir: &str) -> Vec<String> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            files.push(path.to_str().unwrap().to_owned());
        }
    }

    files
}

fn run_tests_from_directory(dir: &str) {
    let files = list_files_from_directory(dir);

    let total = files.len();
    let mut failed: usize = 0;

    for file in files {
        println!("\nrunning test: {}", file);

        let output = run_bin_as_subproccess(&file);

        if !output.status.success() {
            println!(
                "\x1b[31mtest failed:\x1b[0m {} with status {} and stdout: \n\"\n{}\"",
                file,
                output.status,
                String::from_utf8(output.stdout).unwrap()
            );

            failed += 1;
        }
    }

    println!(
        "\n\ntest result: out for {} tests, {} passed and {} failed.",
        total,
        total - failed,
        failed
    );

    std::process::exit(if failed > 0 { 1 } else { 0 });
}

#[test]
fn test_rvi() {
    run_tests_from_directory("testbins/rv32ui/bin/");
}
