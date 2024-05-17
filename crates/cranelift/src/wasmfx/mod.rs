/// This module provides two translations from wasmfx instructions to CLIF. A
/// "baseline" implementation, relying almost entirely on libcalls, and an
/// "optimized" translation, generating more code directly. They are found in
/// the corresponding modules.
///
/// The (private) `shared` module contains some logic shared by both
/// implementations.
mod shared;

#[allow(dead_code)]
pub(crate) mod baseline;

#[allow(dead_code)]
pub(crate) mod optimized;
