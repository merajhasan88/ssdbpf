# ssdbpf

An eBPF runtime for NVMe computational storage, written in Rust.

**The aim:** an interpreter, a verifier and a JIT compiler small enough to run on an SSD controller, so that a
host can send eBPF programs to the drive and have them run next to the data. This follows the NVMe Computational
Programs command set (TP4091).

**Status: early development.** Nothing here runs eBPF yet. The project is being built in stages, and this repository
currently holds stage 0.

## Stage 0: RISC-V warm-up (in progress)

`stage0/` builds the two pieces a JIT needs, on 64-bit RISC-V Linux:

- **Instruction encoders** (`src/encode.rs`): pure functions that turn instructions such as `add` into 32-bit
  machine words. Registers and the opcode, funct3 and funct7 fields are range-checked, and errors come back as a
  `Result`. They are tested against the encodings LLVM's assembler produces.
- **Running code from memory** (`src/exec.rs`, not written yet): map a page, write the words, make it executable, and
  call it.

## Build and test

You need a Rust toolchain with edition 2024 support (developed on Rust 1.100).

Tests on your own machine:

```sh
cd stage0
cargo test
```

Tests on RISC-V, under QEMU user mode. On Ubuntu 22.04 the packages are `gcc-riscv64-linux-gnu`,
`libc6-dev-riscv64-cross` and `qemu-user`:

```sh
rustup target add riscv64gc-unknown-linux-gnu
cd stage0
cargo test --target riscv64gc-unknown-linux-gnu
```

`stage0/.cargo/config.toml` sets the linker, static linking and the QEMU runner. Builds print linker warnings about
`getpwuid_r` and `getaddrinfo` in static binaries; they are harmless here.

While stage 0 is in progress:
- `cargo run` on your own machine stops with a message asking for the RISC-V target.
- On RISC-V, `cargo run` stops at `not yet implemented`.

## Plan

1. Stage 0: RISC-V encoders and running code from memory
2. Stage 1: an eBPF interpreter, checked against the bpf_conformance test suite
3. Stage 2: differential testing against other eBPF implementations
4. Stage 3: a JIT from eBPF to 64-bit RISC-V, under QEMU
5. Stage 4: a JIT for 32-bit RISC-V, on a Raspberry Pi Pico 2 (RP2350)

After that: memory bounds checks, an instruction budget, more backends, and an emulated NVMe device.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in this work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
