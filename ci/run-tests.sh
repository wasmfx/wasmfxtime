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
#
# - veri_engine: requires an SMT solver (z3)

cargo test \
      --workspace \
      --features=default \
      --exclude test-programs \
      --exclude wasmtime-wasi-nn \
      --exclude wasmtime-fuzzing \
      --exclude wasm-spec-interpreter \
      --exclude wasmtime-winch \
      --exclude veri_engine \
      $@

# NOTE(dhil): Several WasmFX features are conflicting, so we do not
# want to run with `--all-features`.
# cargo test \
#       --workspace \
#       --all-features \
#       --exclude test-programs \
#       --exclude wasmtime-wasi-nn \
#       --exclude wasmtime-fuzzing \
#       --exclude wasm-spec-interpreter \
#       --exclude veri_engine \
#       $@
