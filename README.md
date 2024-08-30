# RISCVBox: A high-performance RISC-V emulator, JIT-compiled in Rust

[![Rust CI/CD](https://github.com/bane9/RISCVBox/actions/workflows/rust.yml/badge.svg)](https://github.com/bane9/RISCVBox/actions/workflows/rust.yml)

Welcome to the RISCVBox repository! It hosts the source code for the RISC-V box emulator—a rv32ima systems emulator enabling Linux boot by translating the environment to x86_64 assembly.

https://github.com/user-attachments/assets/d7f1de3c-6000-446b-b712-9fc4c8e2bc29

## Table of contents
- [Features](#features)
- [Building](#building)
- [Usage](#usage)
- [Building RISC-V Linux](#building-risc-v-linux)
- [Testing](#testing)
- [Supported Platforms](#supported-platforms)
- [License](#license)

## Features
- RV32IMASU RISC-V frontend
- x86_64 JIT backend
- SV32 MMU
- ASID aware TLB
- Peripherals:
    - PLIC
    - CLINT
    - NS16550A
    - RAMFB
    - SYSCON

## Building
First, install [rustup](https://rustup.rs/), then clone this project:
```bash
git clone https://github.com/bane9/RISCVBox.git
```

Inside the repository, set the compiler to the nighly version
```bash
rustup override set nightly
```

Then build the project
```bash
cargo b --release
```

This repository holds prebuilt Linux and OpenSBI binaries you can test out. To run them, do the following
```bash
cargo r --release -- --bios linux/prebuilt/fw_jump.bin --kernel linux/prebuilt/Image
```

## Usage
```
Usage: RISCVBox.exe [OPTIONS] --bios <BIOS>

Options:
  -b, --bios <BIOS>      Path to BIOS (firmware) image
  -k, --kernel <KERNEL>  Path to Linux kernel image [default: ]
  -m, --memory <MEMORY>  Memory size in MiB [default: 64]
      --nographic        Disable the graphical output (only output to console)
      --width <WIDTH>    Width of the graphical output in pixels [default: 800]
      --height <HEIGHT>  Height of the graphical output in pixels [default: 600]
  -s, --scale <SCALE>    Scale factor for the graphical output (1, 2, 4, 8, 16, 32) [default: 1]
  -h, --help             Print help
  -V, --version          Print version
```

## Building RISC-V Linux
This reposotory provides Buildroot configuration files to enable building of the Linux kernel and OpenSBI bootloader with configuration that are compatible with this emulator.

To utilize them, do the following (Linux only):
```bash
sudo apt install -y git build-essential wget cpio unzip rsync bc libncurses5-dev screen bison file flex

cd linux
./build.sh
```

The bootloader (fw_jump.bin) and the kernel (Image) will end up in `linux/output` folder

## Testing

The emulator provides unit testing for amd64 instruction generation as well for [riscv-tests](https://github.com/riscv-software-src/riscv-tests).

To run the instruction unit tests, do the following:
```bash
cargo test RISCVBox
```

If you wan't to utlize riscv-tests, you need to build them first:

First, init the riscv-test submodule
```
git submodule init riscv-tests
```

Then, install the riscv gnu toolchain:

**Linux**

```bash
sudo apt install cmake gcc-riscv32-unknown-elf
```

**Mac**

```bash
brew tap riscv-software-src/riscv
brew install riscv-tools cmake
```

Then, build the test via:
```bash
cmake -P misc/MakeTests.cmake
```

Finally, execute one of them like this:
```bash
cargo test --release test_rvi
```

The full list is: `test_rvi test_rvm test_rva test_rvmi test_rvsi`

## Supported platforms

| Platform        | Compatible | Comments                           |
|-----------------|------------|------------------------------------|
| Windows amd64   | ✅         |                                    |
| Ubuntu amd64    | ✅         |                                    |
| MacOS amd64     | ❌         | Compiles, but MAP_FIXED seems to be unsupported on newer MacOS' |

## Note

This emulator lacks internal safeguards against potential malicious payloads that could result in arbitrary code execution. It is strongly recommended to run only trusted software.

## License

This repository is under GNU GPLv3 license.

The RISC-V trade name is a registered trade mark of RISC-V International.
