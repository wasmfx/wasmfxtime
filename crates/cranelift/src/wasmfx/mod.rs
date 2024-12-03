/// This module provides two translations from wasmfx instructions to CLIF. A
/// "baseline" implementation, relying almost entirely on libcalls, and an
/// "optimized" translation, generating more code directly. They are found in
/// the corresponding modules.
///
/// The (private) `shared` module contains some logic shared by both
/// implementations.
mod shared;

#[cfg_attr(
    any(not(feature = "wasmfx_baseline"), feature = "wasmfx_no_baseline"),
    allow(dead_code, reason = "TODO")
)]
pub(crate) mod baseline;

#[cfg_attr(feature = "wasmfx_baseline", allow(dead_code, reason = "TODO"))]
pub(crate) mod optimized;
