use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    set_commit_info_for_rustc();
    select_wasmfx_implementation();
}

fn set_commit_info_for_rustc() {
    if !Path::new(".git").exists() {
        return;
    }
    let output = match Command::new("git")
        .arg("log")
        .arg("-1")
        .arg("--date=short")
        .arg("--format=%H %h %cd")
        .arg("--abbrev=9")
        .output()
    {
        Ok(output) if output.status.success() => output,
        _ => return,
    };
    let stdout = String::from_utf8(output.stdout).unwrap();
    let mut parts = stdout.split_whitespace();
    let mut next = || parts.next().unwrap();
    println!("cargo:rustc-env=WASMTIME_GIT_HASH={}", next());
    println!(
        "cargo:rustc-env=WASMTIME_VERSION_INFO={} ({} {})",
        env!("CARGO_PKG_VERSION"),
        next(),
        next()
    );
}

// NOTE(dhil): This is a workaround the fact that cargo features are
// additive. Having `wasmfx_baseline` as a feature in Cargo.toml means
// it always overrides the main development aka optimized version of
// our implementation when running in the CI.
fn select_wasmfx_implementation() {
    println!("cargo:rerun-if-env-changed=WASMFX_IMPL");
    match std::env::var("WASMFX_IMPL") {
        Ok(val) if &val == "baseline" => {
            println!("cargo:rustc-cfg=feature=\"wasmfx_baseline\"")
        }
        _ => {}
    }
}
