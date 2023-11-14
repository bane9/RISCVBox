#![allow(dead_code)]

mod backend;
mod bus;
mod cpu;
mod frontend;
mod util;
mod xmem;

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

    fn write(&mut self, _addr: BusType, data: BusType, _size: BusType) -> Result<(), Exception> {
        eprintln!("tohost: {:#x}", data);

        if data == 1 {
            std::process::exit(0);
        } else {
            std::process::exit(1);
        }
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

fn main() {
    let ram_size = 16 * 4096;

    let argv = std::env::args().collect::<Vec<String>>();
    let rom = util::read_file(&argv[1]).unwrap();

    init_bus(rom.clone(), ram_size);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    exec_thread_pool.join();
}

fn run_bin_as_subproccess(bin: &str) -> Output {
    let child = std::process::Command::new("target/debug/deps/test_riscv_isa")
        .arg(bin)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
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

    let mut failed = false;

    for file in files {
        println!("\nrunning test: {}", file);

        let output = run_bin_as_subproccess(&file);

        if !output.status.success() {
            println!(
                "\x1b[31mtest failed:\x1b[0m {} with status {} and stderr: \"{}\"",
                file,
                output.status,
                String::from_utf8(output.stderr).unwrap()
            );
            failed = true;
        }
    }

    std::process::exit(if failed { 1 } else { 0 });
}

#[test]
fn test_rvi() {
    run_tests_from_directory("testbins/rv32ui/bin/");
}
