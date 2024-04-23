#![allow(dead_code)]
#![feature(thread_local)]
#![allow(unused_imports)]
#[allow(arithmetic_overflow)]
mod backend;
mod bus;
mod cpu;
mod frontend;
mod util;
mod window;
mod xmem;

use std::io::Write;
use std::thread;
use std::time::Duration;

use backend::csr::init_backend_csr;
use bus::{ram::RAM_BEGIN_ADDR, BusType};
use cpu::Exception;
use frontend::exec_core::ExecCoreThreadPool;
use std::path::PathBuf;
use std::process::Output;

const TOHOSTADDR: BusType = 0x01000000;

struct ToHost;

impl bus::BusDevice for ToHost {
    fn load(&mut self, _addr: BusType, _size: BusType) -> Result<BusType, Exception> {
        Ok(0)
    }

    fn store(&mut self, addr: BusType, _data: BusType, _size: BusType) -> Result<(), Exception> {
        if addr >= TOHOSTADDR + 4 {
            // Ignore fromhost
            return Ok(());
        }

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
        TOHOSTADDR + 16
    }

    fn get_ptr(&mut self, _addr: BusType) -> Result<*mut u8, Exception> {
        Ok(std::ptr::null_mut())
    }

    fn tick_core_local(&mut self) {}
    fn tick_from_main_thread(&mut self) {}
    fn tick_async(&mut self, _cpu: &mut cpu::Cpu) -> Option<u32> {
        None
    }

    fn describe_fdt(&self, _fdt: &mut vm_fdt::FdtWriter) {}
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
    let ram_size: usize = util::size_kib(64);

    let argv = std::env::args().collect::<Vec<String>>();

    if argv.len() < 2 {
        println!("Usage: {} <bin> [timeout]", argv[0]);
        std::process::exit(1);
    }

    let rom = util::read_file(&argv[1]).unwrap();

    if argv.len() == 3 && argv[2] == "timeout" {
        timeout_thread();
    }

    init_backend_csr();

    init_bus(rom.clone(), ram_size);

    let exec_thread_pool = ExecCoreThreadPool::new(RAM_BEGIN_ADDR, 1);

    exec_thread_pool.join();

    std::process::exit(1); // It's only valid to exit from the tohost device
}

fn get_least_one_file(files: &[&str]) -> Option<String> {
    for file in files {
        if std::path::Path::new(file).exists() {
            return Some(file.to_string());
        }
    }

    None
}

fn run_bin_as_subproccess(bin: &PathBuf) -> Output {
    let path = get_least_one_file(&[
        "target/release/test_riscv_isa",
        "target/release/test_riscv_isa.exe",
    ]);

    if path.is_none() {
        panic!("Please compile test_riscv_isa as release first")
    }

    let path = path.unwrap();
    let path = PathBuf::from(path);

    let child = std::process::Command::new(path)
        .arg(bin)
        .arg("timeout")
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::inherit())
        .output()
        .unwrap();

    child
}

fn list_files_from_directory(dir: &str) -> Vec<PathBuf> {
    let mut files = Vec::new();

    for entry in std::fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_file() {
            files.push(PathBuf::from(path.to_str().unwrap()));
        }
    }

    files
}

fn run_tests_from_directory(dir: &str, skip_list: &[&str]) {
    let files = list_files_from_directory(dir);

    let total = files.len();
    let mut failed: usize = 0;

    'test: for file in files {
        let file_str = file.as_path().to_str().unwrap();

        for skip in skip_list {
            if file_str.contains(skip) {
                println!("\nskipping test: {:}", file_str);
                continue 'test;
            }
        }

        println!("\nrunning test: {:}", file_str);

        let output = run_bin_as_subproccess(&file);

        if !output.status.success() {
            println!(
                "\x1b[31mtest failed:\x1b[0m {} with status {} and stdout: \n\"\n{}\"",
                file_str,
                output.status,
                String::from_utf8(output.stdout).unwrap()
            );

            failed += 1;
        }
    }

    println!(
        "\n\nTest result: out of {} tests, {} passed and {} failed.",
        total,
        total - failed,
        failed
    );

    std::process::exit(if failed > 0 { 1 } else { 0 });
}

const NOSKIP: &[&str] = &[];

#[test]
fn test_rvi() {
    run_tests_from_directory("testbins/rv32ui/bin/", NOSKIP);
}

#[test]
fn test_rvm() {
    run_tests_from_directory("testbins/rv32um/bin/", NOSKIP);
}

#[test]
fn test_rva() {
    run_tests_from_directory("testbins/rv32ua/bin/", NOSKIP);
}

#[test]
fn test_rvmi() {
    run_tests_from_directory("testbins/rv32mi/bin/", NOSKIP);
}

#[test]
fn test_rvsi() {
    // Dirty bit generally won't always be set because of the TLB
    run_tests_from_directory("testbins/rv32si/bin/", &["dirty"]);
}
