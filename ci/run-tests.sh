#!/usr/bin/env bash

# Excludes:
#
# - test-programs: just programs used in tests.
#
# - wasmtime-wasi-nn: mutually-exclusive features that aren't available for all
#   targets, needs its own CI job.
#
# - wasmtime-fuzzing: enabling all features brings in OCaml which is a pain to
#   configure for all targets, so it has its own CI job.
#
# - wasm-spec-interpreter: brings in OCaml which is a pain to configure for all
#   targets, tested as part of the wastime-fuzzing CI job.

cargo test \
      --workspace \
      --features=all-arch,winch,wmemcheck,disable-logging,wasi-nn,wasi-threads,wasi-http,pooling-allocator,component-model,wat,cache,parallel-compilation,logging,demangle,cranelift,profiling,coredump,addr2line,debug-builtins,threads,gc,old-cli,serve,explore,wast,config,compile,run \
      --exclude test-programs \
      --exclude wasmtime-wasi-nn \
      --exclude wasmtime-fuzzing \
      --exclude wasm-spec-interpreter \
      $@

      # NOTE(dhil): Several WasmFX "features" aren't additive, so we
      # don't want to run `--all-features` as it would inadvertently
      # toggle a conflicting set of features.
# cargo test \
#       --workspace \
#       --all-features \
#       --exclude test-programs \
#       --exclude wasmtime-wasi-nn \
#       --exclude wasmtime-fuzzing \
#       --exclude wasm-spec-interpreter \
#       $@
