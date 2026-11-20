//! Executable memory on Linux, RISC-V 64.
//! Linux-specific: on the Pico 2 (no OS) this module gets rewritten; `encode` does not.

use core::ffi::c_void;

// TODO: declare the C functions you need (edition 2024 wants `unsafe extern "C"`):
//   mmap, mprotect, munmap   -> man 2 mmap, man 2 mprotect
//   __riscv_flush_icache     -> /usr/riscv64-linux-gnu/include/sys/cachectl.h
// Constants PROT_* and MAP_*  -> /usr/riscv64-linux-gnu/include/bits/mman-linux.h
//                                and bits/mman-map-flags-generic.h
// (Or add the `libc` crate to Cargo.toml and use its declarations instead.)
unsafe extern "C" {}

/// What can go wrong. Consider carrying `std::io::Error::last_os_error()` in each variant.
#[derive(Debug)]
pub enum ExecError {
    Map,
    Protect,
    Flush,
    // Is empty `code` an error? Decide.
}

/// Generated code in memory: writable while it is filled, then executable only (never both).
pub struct ExecBuf {
    ptr: *mut c_void,
    len: usize,
}

impl ExecBuf {
    /// The task's steps: map writable memory, copy the words in,
    /// make it executable (not writable), sync the instruction cache.
    pub fn new(code: &[u32]) -> Result<Self, ExecError> {
        todo!()
    }

    /// Calls the code as `extern "C" fn(u64, u64) -> u64`.
    ///
    /// # Safety
    /// TODO: what must the caller guarantee about the code in this buffer?
    pub unsafe fn call2(&self, a: u64, b: u64) -> u64 {
        todo!()
    }
}

impl Drop for ExecBuf {
    fn drop(&mut self) {
        // TODO: release the mapping. `drop` cannot return an error: what will you do if it fails?
    }
}
