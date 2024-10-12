## What is this?
Baby steps towards running Rust on a Feather RP2040 development board. This project describes how to setup a minimal embedded project. If you want to blink an LED, well, this repo is not that.

## Standard Rust tooling
- [rustup](https://www.rust-lang.org/tools/install) : Rust toolchain manager
- `rustup update`
- `rustup --version`
- Installing Rust using `rustup` will also install `cargo` (package manager) and `rustc` (compiler)

## Embedded tooling
- GNU Debugger : `brew install arm-none-eabi-gdb`
- Open On-Chip Debugger : `brew install openocd`

## Toolchain support for the ARM Cortex-M0+ processors
- [thumbv6m-none-eabi](https://doc.rust-lang.org/nightly/rustc/platform-support/thumbv6m-none-eabi.html)
- `rustup target add thumbv6m-none-eabi`

## Runner: Loading a UF2-file over USB
- `cargo install elf2uf2-rs`

## cargo-binutils
- `cargo install cargo-binutils`
- `rustup component add llvm-tools-preview`

## Create new project
- Create repo at GitHub
- Clone repo
- `cd minimal-embedded-rust`
- `cargo init`
- Check for errors (fast, doesn't produce executable) : `cargo check`
- Build (will create folder *target*) : `cargo build`
- Run : `./target/debug/rust-feather-rp2040`
- Build for release: `cargo build --release`
- Build and run in one step : `cargo run`

## Add readme file
- `touch README.md`
- `subl README.md` or `vim README.md`
- `git status`
- `git add README.md`
- `git commit -m "add readme"`

## Hello world
- `bat ./src/main.rs`
```rust
fn main() {
  println!("Hello, world!");
}
```

## Embedded : no_std & no_main
- `no_std` : An embedded program will use the `core` crate, a subset of `std` that can run on bare metal systems
- `no_main` : Instead of the standard `main` we'll use the entry attribute from the `cortex-m-rt` crate to define a custom entry point
- The entry point function must have signature `fn() -> !` which indicates that the function can't return (the program never terminates)

## Minimal embedded main.rs
- `vim src/main.rs`
```rust
#![no_main]
#![no_std]
use cortex_m_rt::entry;
use panic_halt as _;
#[entry]
fn main() -> ! {
  loop{}
}
```

## Add dependencies
- `vim Cargo.toml`
```toml
[dependencies]
cortex-m = "0.7.7"
cortex-m-rt = "0.7.3"
panic-halt = "0.2.0"
```

## Disabling stack unwinding
- `vim Cargo.toml`
```toml
# profile used for `cargo build`
[profile.dev]
panic = "abort" # disable stack unwinding on panic

# profile used for `cargo build --release`
[profile.release]
panic = "abort" # disable stack unwinding on panic
```

## Add target
- By default Rust tries to build an executable that is able to run in your current system environment (host)
- `rustc --version --verbose`
```terminal
rustc 1.81.0 (eeb90cda1 2024-09-04)
binary: rustc
commit-hash: eeb90cda1969383f56a2637cbd3037bdf598841c
commit-date: 2024-09-04
host: aarch64-apple-darwin
release: 1.81.0
LLVM version: 18.1.7
```
- `mkdir .cargo`
- `touch .cargo/config.toml`
- `vim .cargo/config.toml`
```toml
[build]
target = "thumbv6m-none-eabi"
```

## Add target memory layout
- The Feather RP2040 has 8 MB SPI FLASH
- 1024 * 8 = 8192
- The Feather RP2040 has 264 KB RAM
- `touch memory.x`
- `vim memory.x`
```
MEMORY {
    BOOT2 : ORIGIN = 0x10000000, LENGTH = 0x100
    FLASH : ORIGIN = 0x10000100, LENGTH = 8192K - 0x100
    RAM   : ORIGIN = 0x20000000, LENGTH = 264K
}

SECTIONS {
    /* ### Boot loader */
    .boot2 ORIGIN(BOOT2) :
    {
        KEEP(*(.boot2));
    } > BOOT2
} INSERT BEFORE .text;
```

## Sanity check
- Let's verify if the source code can produce an ARM binary executable
- Install `cargo-binutils` if you haven't already
- `cargo readobj --target thumbv6m-none-eabi --bin minimal-embedded-rust -- --file-headers`
- This will implicitly run `cargo build`
- Here we are looking at Machine = ARM (OK)
```terminal
ELF Header:
  Magic:   7f 45 4c 46 01 01 01 00 00 00 00 00 00 00 00 00
  Class:                             ELF32
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           ARM
  Version:                           0x1
  Entry point address:               0x0
  Start of program headers:          52 (bytes into file)
  Start of section headers:          667472 (bytes into file)
  Flags:                             0x5000200
  Size of this header:               52 (bytes)
  Size of program headers:           32 (bytes)
  Number of program headers:         3
  Size of section headers:           40 (bytes)
  Number of section headers:         14
  Section header string table index: 12
```

## Configure linker
- In the sanity check above we got that the **entry point address** is 0x0 (NOK)
- If we try to flash the target with `cargo run` we will get the following error : *Error: "entry point is not in mapped part of file"*
- To fix this issue we need to configure the linker
- Linker argument `-Tlink.x` tells the linker to use `link.x` as the linker script. This is usually provided by the `cortex-m-rt` crate, and by default the version in that crate will include a file called `memory.x` which describes the particular memory layout for your specific chip
- `vim .cargo/config.toml`
```toml
[target.thumbv6m-none-eabi]
rustflags = [
  "-C", "link-arg=-Tlink.x",
]
```
- `cargo readobj --target thumbv6m-none-eabi --bin test-feather -- --file-headers`
```
ELF Header:
  Magic:   7f 45 4c 46 01 01 01 00 00 00 00 00 00 00 00 00
  Class:                             ELF32
  Data:                              2's complement, little endian
  Version:                           1 (current)
  OS/ABI:                            UNIX - System V
  ABI Version:                       0
  Type:                              EXEC (Executable file)
  Machine:                           ARM
  Version:                           0x1
  Entry point address:               0x100001C1
  Start of program headers:          52 (bytes into file)
  Start of section headers:          734688 (bytes into file)
  Flags:                             0x5000200
  Size of this header:               52 (bytes)
  Size of program headers:           32 (bytes)
  Number of program headers:         3
  Size of section headers:           40 (bytes)
  Number of section headers:         21
  Section header string table index: 19
```

## Add runner
- This runner will copy an UF2 file to a mounted RP2040 in USB
- `vim .cargo/config.toml`
```toml
[target.thumbv6m-none-eabi]
runner = "elf2uf2-rs -d"
```

## Flash target
- Inside the RP2040 is a 'permanent ROM' USB UF2 bootloader
- What that means is when you want to program new firmware, you can hold down the BOOTSEL button while plugging it into USB and it will appear as a USB disk drive you can drag the firmware onto (or run `cargo run`)

## Resources
- [The Embedded Rust Book](https://docs.rust-embedded.org/book/intro/index.html)
- [Rust Discovery Book](https://docs.rust-embedded.org/discovery/microbit/index.html)
- [cortex-m](https://crates.io/crates/cortex-m)
- [cortex-m-rt](https://crates.io/crates/cortex-m-rt)
- [panic-halt](https://crates.io/crates/panic-halt)
- ["Hello world!" without Standard Library](https://zenn.dev/zulinx86/articles/rust-nostd-101)
- [cargo-binutils](https://crates.io/crates/cargo-binutils)
- [Sanity check](https://docs.rust-embedded.org/discovery/microbit/05-led-roulette/build-it.html)
- [Add memory layout](https://docs.rs/cortex-m-rt/0.7.3/cortex_m_rt/)
- [Adafruit Feather RP2040](https://www.adafruit.com/product/4884)

## Useful repos
- [rp-hal](https://github.com/rp-rs/rp-hal)
- [Blinky test in Rust for NRF32-DK](https://gist.github.com/BlinkingApe/9b4f5202c0294ce47a883633fc94e71b)
- [rp-hal-board](https://github.com/rp-rs/rp-hal-boards)
- [discovery](https://github.com/rust-embedded/discovery)
- [rp2040-project-template](https://github.com/rp-rs/rp2040-project-template)
- [cortex-m](https://github.com/rust-embedded/cortex-m)
