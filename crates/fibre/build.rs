fn main() {
    // assume that this is included via inline assembly in the crate itself,
    // and the crate will otherwise have a `compile_error!` for unsupported
    // platforms.
    println!("cargo:rerun-if-changed=build.rs");
    return;
}
