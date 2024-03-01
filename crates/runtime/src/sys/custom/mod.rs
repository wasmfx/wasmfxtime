//! Custom platform support in Wasmtime.
//!
//! This module contains an implementation of defining Wasmtime's platform
//! support in terms of a minimal C API. This API can be found in the `capi`
//! module and all other functionality here is implemented in terms of that
//! module.
//!
//! For more information about this see `./examples/min-platform` as well as
//! `./docs/examples-minimal.md`.

use std::io;

pub mod capi;
pub mod mmap;
pub mod traphandlers;
pub mod unwind;
pub mod vm;

fn cvt(rc: i32) -> io::Result<()> {
    match rc {
        0 => Ok(()),
        code => Err(io::Error::from_raw_os_error(code)),
    }
}
