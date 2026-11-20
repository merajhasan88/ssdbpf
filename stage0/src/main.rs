//! Task 0: run machine code that we wrote into memory.
//!
//! Part A: hand-copied words from llvm-mc.  Part B: words built by `encode`.
//! Part C: no panics, every `unsafe` explained.

#![warn(
    clippy::undocumented_unsafe_blocks,
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic
)]

mod encode;
// Linux + RISC-V only. Gated so plain `cargo test` on your laptop still builds the encoder tests.
#[cfg(target_arch = "riscv64")]
mod exec;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("error: {msg}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(target_arch = "riscv64")]
fn run() -> Result<(), String> {
    // Part A: paste the four words here (from llvm-mc-14 output).
    // Part B: build them with the functions in `encode` instead.
    let code: &[u32] = &[
        // TODO
    ];

    let buf = exec::ExecBuf::new(code).map_err(|e| format!("{e:?}"))?;

    // SAFETY: TODO — why is calling this code sound? (see the `# Safety` section on `call2`)
    let result = unsafe { buf.call2(40, 2) };

    println!("40 + 2 = {result}");
    Ok(())
}

#[cfg(not(target_arch = "riscv64"))]
fn run() -> Result<(), String> {
    Err("this runs generated RISC-V code: use --target riscv64gc-unknown-linux-gnu".into())
}
