use anyhow::{bail, Context};
use libtest_mimic::{Arguments, FormatSetting, Trial};
use std::sync::{Condvar, LazyLock, Mutex};
use wasmtime::{
    Config, Engine, InstanceAllocationStrategy, MpkEnabled, PoolingAllocationConfig, Store,
};
use wasmtime_environ::Memory;
use wasmtime_wast::{SpectestConfig, WastContext};
use wasmtime_wast_util::{limits, Collector, Compiler, WastConfig, WastTest};

fn main() {
    env_logger::init();

    let tests = if cfg!(miri) {
        Vec::new()
    } else {
        wasmtime_wast_util::find_tests(".".as_ref()).unwrap()
    };

    let mut trials = Vec::new();

    // For each test generate a combinatorial matrix of all configurations to
    // run this test in.
    for test in tests {
        let test_uses_gc_types = test.test_uses_gc_types();
        for compiler in [Compiler::Cranelift, Compiler::Winch] {
            for pooling in [true, false] {
                let collectors: &[_] = if !pooling && test_uses_gc_types {
                    &[Collector::DeferredReferenceCounting, Collector::Null]
                } else {
                    &[Collector::Auto]
                };

                for collector in collectors.iter().copied() {
                    let trial = Trial::test(
                        format!(
                            "{compiler:?}/{}{}{}",
                            if pooling { "pooling/" } else { "" },
                            if collector != Collector::Auto {
                                format!("{collector:?}/")
                            } else {
                                String::new()
                            },
                            test.path.to_str().unwrap()
                        ),
                        {
                            let test = test.clone();
                            move || {
                                run_wast(
                                    &test,
                                    WastConfig {
                                        compiler,
                                        pooling,
                                        collector,
                                    },
                                )
                                .map_err(|e| format!("{e:?}").into())
                            }
                        },
                    );
                    trials.push(trial);
                }
            }
        }
    }

    // There's a lot of tests so print only a `.` to keep the output a
    // bit more terse by default.
    let mut args = Arguments::from_args();
    if args.format.is_none() {
        args.format = Some(FormatSetting::Terse);
    }
    libtest_mimic::run(&args, trials).exit()
}

// Each of the tests included from `wast_testsuite_tests` will call this
// function which actually executes the `wast` test suite given the `strategy`
// to compile it.
fn run_wast(test: &WastTest, config: WastConfig) -> anyhow::Result<()> {
    let mut test_config = test.config.clone();

    // FIXME: this is a bit of a hack to get Winch working here for now. Winch
    // passes some tests on aarch64 so returning `true` from `should_fail`
    // doesn't work. Winch doesn't pass many tests though as it either panics or
    // segfaults as AArch64 support isn't finished yet. That means that we can't
    // have, for example, an allow-list of tests that should pass and assume
    // everything else fails. In lieu of all of this we feign all tests as
    // requiring references types which Wasmtime understands that Winch doesn't
    // support on aarch64 which means that all tests fail quickly in config
    // validation.
    //
    // Ideally the aarch64 backend for Winch would return a normal error on
    // unsupported opcodes and not segfault, meaning that this would not be
    // needed.
    if cfg!(target_arch = "aarch64") && test_config.reference_types.is_none() {
        test_config.reference_types = Some(true);
    }

    // Determine whether this test is expected to fail or pass. Regardless the
    // test is executed and the result of the execution is asserted to match
    // this expectation. Note that this means that the test can't, for example,
    // panic or segfault as a result.
    //
    // Updates to whether a test should pass or fail should be done in the
    // `crates/wast-util/src/lib.rs` file.
    let should_fail = test.should_fail(&config);

    // Note that all of these proposals/features are currently default-off to
    // ensure that we annotate all tests accurately with what features they
    // need, even in the future when features are stabilized.
    let memory64 = test_config.memory64.unwrap_or(false);
    let custom_page_sizes = test_config.custom_page_sizes.unwrap_or(false);
    let multi_memory = test_config.multi_memory.unwrap_or(false);
    let threads = test_config.threads.unwrap_or(false);
    let gc = test_config.gc.unwrap_or(false);
    let tail_call = test_config.tail_call.unwrap_or(false);
    let extended_const = test_config.extended_const.unwrap_or(false);
    let wide_arithmetic = test_config.wide_arithmetic.unwrap_or(false);
    let test_hogs_memory = test_config.hogs_memory.unwrap_or(false);
    let component_model_more_flags = test_config.component_model_more_flags.unwrap_or(false);
    let nan_canonicalization = test_config.nan_canonicalization.unwrap_or(false);
    let relaxed_simd = test_config.relaxed_simd.unwrap_or(false);

    // Some proposals in wasm depend on previous proposals. For example the gc
    // proposal depends on function-references which depends on reference-types.
    // To avoid needing to enable all of them at once implicitly enable
    // downstream proposals once the end proposal is enabled (e.g. when enabling
    // gc that also enables function-references and reference-types).
    let stack_switching = test_config.stack_switching.unwrap_or(false);
    let function_references = test_config
        .function_references
        .or(test_config.gc)
        .or(test_config.stack_switching)
        .unwrap_or(false);
    let reference_types = test_config
        .reference_types
        .or(test_config.function_references)
        .or(test_config.gc)
        .unwrap_or(false);
    let exceptions = test_config
        .exceptions
        .or(test_config.stack_switching)
        .unwrap_or(false);

    let is_cranelift = match config.compiler {
        Compiler::Cranelift => true,
        _ => false,
    };

    let mut cfg = Config::new();
    cfg.wasm_multi_memory(multi_memory)
        .wasm_threads(threads)
        .wasm_memory64(memory64)
        .wasm_function_references(function_references)
        .wasm_gc(gc)
        .wasm_reference_types(reference_types)
        .wasm_relaxed_simd(relaxed_simd)
        .wasm_tail_call(tail_call)
        .wasm_custom_page_sizes(custom_page_sizes)
        .wasm_extended_const(extended_const)
        .wasm_wide_arithmetic(wide_arithmetic)
        .wasm_component_model_more_flags(component_model_more_flags)
        .wasm_exceptions(exceptions)
        .wasm_stack_switching(stack_switching)
        .strategy(match config.compiler {
            Compiler::Cranelift => wasmtime::Strategy::Cranelift,
            Compiler::Winch => wasmtime::Strategy::Winch,
        })
        .collector(match config.collector {
            Collector::Auto => wasmtime::Collector::Auto,
            Collector::Null => wasmtime::Collector::Null,
            Collector::DeferredReferenceCounting => wasmtime::Collector::DeferredReferenceCounting,
        })
        .cranelift_nan_canonicalization(nan_canonicalization);

    if is_cranelift {
        cfg.cranelift_debug_verifier(true);
    }

    // By default we'll allocate huge chunks (6gb) of the address space for each
    // linear memory. This is typically fine but when we emulate tests with QEMU
    // it turns out that it causes memory usage to balloon massively. Leave a
    // knob here so on CI we can cut down the memory usage of QEMU and avoid the
    // OOM killer.
    //
    // Locally testing this out this drops QEMU's memory usage running this
    // tests suite from 10GiB to 600MiB. Previously we saw that crossing the
    // 10GiB threshold caused our processes to get OOM killed on CI.
    if std::env::var("WASMTIME_TEST_NO_HOG_MEMORY").is_ok() {
        // The pooling allocator hogs ~6TB of virtual address space for each
        // store, so if we don't to hog memory then ignore pooling tests.
        if config.pooling {
            return Ok(());
        }

        // If the test allocates a lot of memory, that's considered "hogging"
        // memory, so skip it.
        if test_hogs_memory {
            return Ok(());
        }

        // Don't use 4gb address space reservations when not hogging memory, and
        // also don't reserve lots of memory after dynamic memories for growth
        // (makes growth slower).
        cfg.memory_reservation(2 * u64::from(Memory::DEFAULT_PAGE_SIZE));
        cfg.memory_reservation_for_growth(0);

        let small_guard = 64 * 1024;
        cfg.memory_guard_size(small_guard);
    }

    let _pooling_lock = if config.pooling {
        // Some memory64 tests take more than 4gb of resident memory to test,
        // but we don't want to configure the pooling allocator to allow that
        // (that's a ton of memory to reserve), so we skip those tests.
        if test_hogs_memory {
            return Ok(());
        }

        // Reduce the virtual memory required to run multi-memory-based tests.
        //
        // The configuration parameters below require that a bare minimum
        // virtual address space reservation of 450*9*805*65536 == 200G be made
        // to support each test. If 6G reservations are made for each linear
        // memory then not that many tests can run concurrently with much else.
        //
        // When multiple memories are used and are configured in the pool then
        // force the usage of static memories without guards to reduce the VM
        // impact.
        let max_memory_size = limits::MEMORY_SIZE;
        if multi_memory {
            cfg.memory_reservation(max_memory_size as u64);
            cfg.memory_reservation_for_growth(0);
            cfg.memory_guard_size(0);
        }

        let mut pool = PoolingAllocationConfig::default();
        pool.total_memories(limits::MEMORIES * 2)
            .max_memory_protection_keys(2)
            .max_memory_size(max_memory_size)
            .max_memories_per_module(if multi_memory {
                limits::MEMORIES_PER_MODULE
            } else {
                1
            })
            .max_tables_per_module(limits::TABLES_PER_MODULE);

        // When testing, we may choose to start with MPK force-enabled to ensure
        // we use that functionality.
        if std::env::var("WASMTIME_TEST_FORCE_MPK").is_ok() {
            pool.memory_protection_keys(MpkEnabled::Enable);
        }

        cfg.allocation_strategy(InstanceAllocationStrategy::Pooling(pool));
        Some(lock_pooling())
    } else {
        None
    };

    let mut engines = vec![(Engine::new(&cfg), "default")];

    // For tests that use relaxed-simd test both the default engine and the
    // guaranteed-deterministic engine to ensure that both the 'native'
    // semantics of the instructions plus the canonical semantics work.
    if relaxed_simd {
        engines.push((
            Engine::new(cfg.relaxed_simd_deterministic(true)),
            "deterministic",
        ));
    }

    for (engine, desc) in engines {
        let result = engine.and_then(|engine| {
            let store = Store::new(&engine, ());
            let mut wast_context = WastContext::new(store);
            wast_context.register_spectest(&SpectestConfig {
                use_shared_memory: true,
                suppress_prints: true,
            })?;
            wast_context
                .run_buffer(test.path.to_str().unwrap(), test.contents.as_bytes())
                .with_context(|| format!("failed to run spec test with {desc} engine"))
        });

        if should_fail {
            if result.is_ok() {
                bail!("this test is flagged as should-fail but it succeeded")
            }
        }
    }

    Ok(())
}

// The pooling tests make about 6TB of address space reservation which means
// that we shouldn't let too many of them run concurrently at once. On
// high-cpu-count systems (e.g. 80 threads) this leads to mmap failures because
// presumably too much of the address space has been reserved with our limits
// specified above. By keeping the number of active pooling-related tests to a
// specified maximum we can put a cap on the virtual address space reservations
// made.
fn lock_pooling() -> impl Drop {
    const MAX_CONCURRENT_POOLING: u32 = 4;

    static ACTIVE: LazyLock<MyState> = LazyLock::new(MyState::default);

    #[derive(Default)]
    struct MyState {
        lock: Mutex<u32>,
        waiters: Condvar,
    }

    impl MyState {
        fn lock(&self) -> impl Drop + '_ {
            let state = self.lock.lock().unwrap();
            let mut state = self
                .waiters
                .wait_while(state, |cnt| *cnt >= MAX_CONCURRENT_POOLING)
                .unwrap();
            *state += 1;
            LockGuard { state: self }
        }
    }

    struct LockGuard<'a> {
        state: &'a MyState,
    }

    impl Drop for LockGuard<'_> {
        fn drop(&mut self) {
            *self.state.lock.lock().unwrap() -= 1;
            self.state.waiters.notify_one();
        }
    }

    ACTIVE.lock()
}
