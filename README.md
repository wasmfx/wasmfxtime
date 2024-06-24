This repository contains an up-to-date fork of
[Wasmtime](https://docs.wasmtime.dev/), a standalone WebAssembly engine. This
fork adds support for the [WasmFX](https://wasmfx.dev/) proposal for stack
switching. 

The [stack switching repository](https://github.com/WebAssembly/stack-switching) contains a 
[high-level summary of the proposal](https://github.com/WebAssembly/stack-switching/blob/main/proposals/continuations/Explainer.md),
[a more formal description](https://github.com/WebAssembly/stack-switching/blob/main/proposals/continuations/Overview.md),
and [examples](https://github.com/WebAssembly/stack-switching/tree/main/proposals/continuations/examples).

## Building 

The build steps are equivalent to the [standard steps for building Wasmtime from
source](https://docs.wasmtime.dev/contributing-building.html), but using this
repository instead. There is no need to build or install the original version of
Wasmtime to use this fork.

Concretely, the steps are as follows:

1. Make sure that you have a Rust toolchain installed, for example using
   [rustup](https://www.rust-lang.org/tools/install).
2. Check out this repository:
``` sh
git clone https://github.com/wasmfx/wasmfxtime.git
cd wasmfxtime
git submodule update --init
```
3. Build:
``` sh
cargo build
```

As a result, a debug build of the `wasmtime` executable will be created at
`target/debug/wasmtime`.

To create a release build instead, run `cargo build --release`, which will
create `target/release/wasmtime`.


## Running programs

A WebAssembly module `my_module.wat` (or `my_module.wasm`) is executed using the
`wasmtime` executable [in the usual way](https://docs.wasmtime.dev/cli.html). To
run programs containing WasmFX instructions, enable the necessary features as
follows:

``` sh
wasmtime -W=exceptions,function-references,typed-continuations my_module.wat
```

To run an arbitrary function exported as `foo` by the module run 

``` sh
wasmtime -W=exceptions,function-references,typed-continuations --invoke=foo my_module.wat
```

## Example program

The following module implements a generator and consumer using stack switching.

```wat
(module
  (type $ft (func))
  (type $ct (cont $ft))

  ;; Tag used by generator, the i32 payload corresponds to the generated values
  (tag $yield (param i32))

  ;; Printing function for unsigned integers.
  ;; This function is unrelated to stack switching.
  (func $println_u32 (param $value i32)
    ;; See examples/generator.wat for actual implementation
  )


  ;; Simple generator yielding values from 100 down to 1
  (func $generator
    (local $i i32)
    (local.set $i (i32.const 100))
    (loop $l
      ;; Suspend generator, yield current value of $i to consumer
      (suspend $yield (local.get $i))
      ;; Decrement $i and exit loop once $i reaches 0
      (local.tee $i (i32.sub (local.get $i) (i32.const 1)))
      (br_if $l)
    )
  )
  (elem declare func $generator)

  (func $consumer
    (local $c (ref $ct))
    ;; Create continuation executing function $generator
    (local.set $c (cont.new $ct (ref.func $generator)))

    (loop $loop
      (block $on_yield (result i32 (ref $ct))
        ;; Resume continuation $c
        (resume $ct (tag $yield $on_yield) (local.get $c))
        ;; Generator returned: no more data
        (return)
      )
      ;; Generator suspend, stack contains [i32 (ref $ct)]
      (local.set $c)
      ;; Stack now contains the i32 value yielded by generator
      (call $println_u32)

      (br $loop)
    )
  )

  (func $start (export "_start")
    (call $consumer)
  )
)
```

See
[examples/generator.wat](https://github.com/frank-emrich/wasmtime/blob/main/examples/generator.wat)
for the full version of the file, including the definition of `$println_u32`.

Running the full version with 
```
wasmtime -W=exceptions,function-references,typed-continuations generator.wat
```
then prints the numbers 100 down to 1 in the terminal.


## Current limitations

The implementation of the WasmFX proposal is currently limited in a few ways:
- The only supported platform is x64 Linux. 
- Only a single module can be executed. In particular, providing additional
  modules using the `--preload` option of `wasmtime` can lead to unexpected
  behavior.



# Original Wasmtime documentation below

<div align="center">
  <h1><code>wasmtime</code></h1>

  <p>
    <strong>A standalone runtime for
    <a href="https://webassembly.org/">WebAssembly</a></strong>
  </p>

  <strong>A <a href="https://bytecodealliance.org/">Bytecode Alliance</a> project</strong>

  <p>
    <a href="https://github.com/bytecodealliance/wasmtime/actions?query=workflow%3ACI"><img src="https://github.com/bytecodealliance/wasmtime/workflows/CI/badge.svg" alt="build status" /></a>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime"><img src="https://img.shields.io/badge/zulip-join_chat-brightgreen.svg" alt="zulip chat" /></a>
    <img src="https://img.shields.io/badge/rustc-stable+-green.svg" alt="supported rustc stable" />
    <a href="https://docs.rs/wasmtime"><img src="https://docs.rs/wasmtime/badge.svg" alt="Documentation Status" /></a>
  </p>

  <h3>
    <a href="https://bytecodealliance.github.io/wasmtime/">Guide</a>
    <span> | </span>
    <a href="https://bytecodealliance.github.io/wasmtime/contributing.html">Contributing</a>
    <span> | </span>
    <a href="https://wasmtime.dev/">Website</a>
    <span> | </span>
    <a href="https://bytecodealliance.zulipchat.com/#narrow/stream/217126-wasmtime">Chat</a>
  </h3>
</div>

## Installation

The Wasmtime CLI can be installed on Linux and macOS (locally) with a small install
script:

```sh
curl https://wasmtime.dev/install.sh -sSf | bash
```

Windows or otherwise interested users can download installers and
binaries directly from the [GitHub
Releases](https://github.com/bytecodealliance/wasmtime/releases) page.

## Example

If you've got the [Rust compiler
installed](https://www.rust-lang.org/tools/install) then you can take some Rust
source code:

```rust
fn main() {
    println!("Hello, world!");
}
```

and compile/run it with:

```sh
$ rustup target add wasm32-wasi
$ rustc hello.rs --target wasm32-wasi
$ wasmtime hello.wasm
Hello, world!
```

(Note: make sure you installed Rust using the `rustup` method in the official
instructions above, and do not have a copy of the Rust toolchain installed on
your system in some other way as well (e.g. the system package manager). Otherwise, the `rustup target add...`
command may not install the target for the correct copy of Rust.)

## Features

* **Fast**. Wasmtime is built on the optimizing [Cranelift] code generator to
  quickly generate high-quality machine code either at runtime or
  ahead-of-time. Wasmtime is optimized for efficient instantiation, low-overhead
  calls between the embedder and wasm, and scalability of concurrent instances.

* **[Secure]**. Wasmtime's development is strongly focused on correctness and
  security. Building on top of Rust's runtime safety guarantees, each Wasmtime
  feature goes through careful review and consideration via an [RFC
  process]. Once features are designed and implemented, they undergo 24/7
  fuzzing donated by [Google's OSS Fuzz]. As features stabilize they become part
  of a [release][release policy], and when things go wrong we have a
  well-defined [security policy] in place to quickly mitigate and patch any
  issues. We follow best practices for defense-in-depth and integrate
  protections and mitigations for issues like Spectre. Finally, we're working to
  push the state-of-the-art by collaborating with academic researchers to
  formally verify critical parts of Wasmtime and Cranelift.

* **[Configurable]**. Wasmtime uses sensible defaults, but can also be
  configured to provide more fine-grained control over things like CPU and
  memory consumption. Whether you want to run Wasmtime in a tiny environment or
  on massive servers with many concurrent instances, we've got you covered.

* **[WASI]**. Wasmtime supports a rich set of APIs for interacting with the host
  environment through the [WASI standard](https://wasi.dev).

* **[Standards Compliant]**. Wasmtime passes the [official WebAssembly test
  suite](https://github.com/WebAssembly/testsuite), implements the [official C
  API of wasm](https://github.com/WebAssembly/wasm-c-api), and implements
  [future proposals to WebAssembly](https://github.com/WebAssembly/proposals) as
  well. Wasmtime developers are intimately engaged with the WebAssembly
  standards process all along the way too.

[Wasmtime]: https://github.com/bytecodealliance/wasmtime
[Cranelift]: https://cranelift.dev/
[Google's OSS Fuzz]: https://google.github.io/oss-fuzz/
[security policy]: https://bytecodealliance.org/security
[RFC process]: https://github.com/bytecodealliance/rfcs
[release policy]: https://docs.wasmtime.dev/stability-release.html
[Secure]: https://docs.wasmtime.dev/security.html
[Configurable]: https://docs.rs/wasmtime/latest/wasmtime/struct.Config.html
[WASI]: https://docs.rs/wasmtime-wasi/latest/wasmtime_wasi/
[Standards Compliant]: https://docs.wasmtime.dev/stability-tiers.html

## Language Support

You can use Wasmtime from a variety of different languages through embeddings of
the implementation.

Languages supported by the Bytecode Alliance:

* **[Rust]** - the [`wasmtime` crate]
* **[C]** - the [`wasm.h`, `wasi.h`, and `wasmtime.h` headers][c-headers], [CMake](crates/c-api/CMakeLists.txt) or [`wasmtime` Conan package]
* **C++** - the [`wasmtime-cpp` repository][wasmtime-cpp] or use [`wasmtime-cpp` Conan package]
* **[Python]** - the [`wasmtime` PyPI package]
* **[.NET]** - the [`Wasmtime` NuGet package]
* **[Go]** - the [`wasmtime-go` repository]
* **[Ruby]** - the [`wasmtime` gem]

Languages supported by the community:

* **[Elixir]** - the [`wasmex` hex package]
* **Perl** - the [`Wasm` Perl package's `Wasm::Wasmtime`]

[Rust]: https://bytecodealliance.github.io/wasmtime/lang-rust.html
[C]: https://bytecodealliance.github.io/wasmtime/examples-c-embed.html
[`wasmtime` crate]: https://crates.io/crates/wasmtime
[c-headers]: https://bytecodealliance.github.io/wasmtime/c-api/
[Python]: https://bytecodealliance.github.io/wasmtime/lang-python.html
[`wasmtime` PyPI package]: https://pypi.org/project/wasmtime/
[.NET]: https://bytecodealliance.github.io/wasmtime/lang-dotnet.html
[`Wasmtime` NuGet package]: https://www.nuget.org/packages/Wasmtime
[Go]: https://bytecodealliance.github.io/wasmtime/lang-go.html
[`wasmtime-go` repository]: https://pkg.go.dev/github.com/bytecodealliance/wasmtime-go
[wasmtime-cpp]: https://github.com/bytecodealliance/wasmtime-cpp
[`wasmtime` Conan package]: https://conan.io/center/wasmtime
[`wasmtime-cpp` Conan package]: https://conan.io/center/wasmtime-cpp
[Ruby]: https://bytecodealliance.github.io/wasmtime/lang-ruby.html
[`wasmtime` gem]: https://rubygems.org/gems/wasmtime
[Elixir]: https://docs.wasmtime.dev/lang-elixir.html
[`wasmex` hex package]: https://hex.pm/packages/wasmex
[`Wasm` Perl package's `Wasm::Wasmtime`]: https://metacpan.org/pod/Wasm::Wasmtime

## Documentation

[📚 Read the Wasmtime guide here! 📚][guide]

The [wasmtime guide][guide] is the best starting point to learn about what
Wasmtime can do for you or help answer your questions about Wasmtime. If you're
curious in contributing to Wasmtime, [it can also help you do
that][contributing]!

[contributing]: https://bytecodealliance.github.io/wasmtime/contributing.html
[guide]: https://bytecodealliance.github.io/wasmtime

---

It's Wasmtime.
