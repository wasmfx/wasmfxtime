/// This module provides two translations from wasmfx instructions to CLIF. A
/// "baseline" implementation, relying almost entirely on libcalls, and an
/// "optimized" translation, generating more code directly. They are found in
/// the corresponding modules.
///
/// The (private) `shared` module contains some logic shared by both
/// implementations.
mod shared;

#[cfg_attr(
    not(feature = "typed_continuations_baseline_implementation"),
    allow(dead_code)
)]
pub(crate) mod baseline;

#[cfg_attr(
    feature = "typed_continuations_baseline_implementation",
    allow(dead_code)
)]
pub(crate) mod optimized;
