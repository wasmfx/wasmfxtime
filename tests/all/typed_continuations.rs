use anyhow::Result;
use wasmtime::*;

mod test_utils {
    use anyhow::{bail, Result};
    use std::any::*;
    use std::panic::AssertUnwindSafe;
    use wasmtime::*;

    pub struct Runner {
        pub engine: Engine,
        pub store: Store<()>,
    }

    impl Runner {
        pub fn new() -> Runner {
            let mut config = Config::default();
            config.wasm_function_references(true);
            config.wasm_exceptions(true);
            config.wasm_typed_continuations(true);

            let engine = Engine::new(&config).unwrap();

            let store = Store::<()>::new(&engine, ());

            Runner { engine, store }
        }

        /// Uses this `Runner` to run the module defined in `wat`, satisfying
        /// its imports using `imports`. The module must export a function
        /// `entry`, taking no parameters and returning `Results`.
        pub fn run_test<Results: WasmResults>(
            mut self,
            wat: &str,
            imports: &[Extern],
        ) -> Result<Results> {
            let module = Module::new(&self.engine, wat)?;

            let instance = Instance::new(&mut self.store, &module, imports)?;
            let entry = instance.get_typed_func::<(), Results>(&mut self.store, "entry")?;

            entry.call(&mut self.store, ())
        }

        /// Uses this `Runner` to run the module defined in `wat`, satisfying
        /// its imports using `imports`. The module must export a function
        /// `entry`, taking no parameters and without return values . Execution
        /// of `entry` is expected to yield a runtime `Error` with a `&str`
        /// payload (such as an error raised with anyhow::anyhow!("Something is
        /// wrong")
        pub fn run_test_expect_str_error(self, wat: &str, imports: &[Extern], error_message: &str) {
            let result = self.run_test::<()>(wat, imports);

            let err = result.expect_err("Was expecting wasm execution to yield error");

            assert_eq!(err.downcast_ref::<&'static str>(), Some(&error_message));
        }

        /// Uses this `Runner` to run the module defined in `wat`, satisfying
        /// its imports using `imports`. The module must export a function
        /// `entry`, taking no parameters and without return values. Execution
        /// of `entry` is expected to cause a panic (and that this is panic is
        /// not handled by wasmtime previously).
        /// Returns the `Error` payload.
        pub fn run_test_expect_panic(
            mut self,
            wat: &str,
            imports: &[Extern],
        ) -> Box<dyn Any + Send + 'static> {
            let module = Module::new(&self.engine, wat).unwrap();

            let instance = Instance::new(&mut self.store, &module, imports).unwrap();
            let entry = instance.get_func(&mut self.store, "entry").unwrap();

            std::panic::catch_unwind(AssertUnwindSafe(|| {
                drop(entry.call(&mut self.store, &mut [], &mut []))
            }))
            .unwrap_err()
        }
    }

    /// Creates a simple Host function that increments an i32
    pub fn make_i32_inc_host_func(runner: &mut Runner) -> Func {
        Func::new(
            &mut runner.store,
            FuncType::new(&runner.engine, vec![ValType::I32], vec![ValType::I32]),
            |mut _caller, args: &[Val], results: &mut [Val]| {
                let res = match args {
                    [Val::I32(i)] => i + 1,
                    _ => bail!("Error: Received illegal argument (should be single i32)"),
                };
                results[0] = Val::I32(res);
                Ok(())
            },
        )
    }

    /// Creates a host function of type i32 -> i32. `export_func` must denote an
    /// exported function of type i32 -> i32. The created host function
    /// increments its argument by 1, passes it to the exported function, and in
    /// turn increments the result before returning it as the overall result.
    pub fn make_i32_inc_via_export_host_func(
        runner: &mut Runner,
        export_func: &'static str,
    ) -> Func {
        Func::new(
            &mut runner.store,
            FuncType::new(&runner.engine, vec![ValType::I32], vec![ValType::I32]),
            |mut caller, args: &[Val], results: &mut [Val]| {
                let export = caller
                    .get_export(export_func)
                    .ok_or(anyhow::anyhow!("could not get export"))?;
                let func = export
                    .into_func()
                    .ok_or(anyhow::anyhow!("export is not a Func"))?;
                let func_typed = func.typed::<i32, i32>(caller.as_context())?;
                let arg = args[0].unwrap_i32();
                let res = func_typed.call(caller.as_context_mut(), arg + 1)?;
                results[0] = Val::I32(res + 1);
                Ok(())
            },
        )
    }

    pub fn make_panicking_host_func(store: &mut Store<()>, msg: &'static str) -> Func {
        Func::wrap(store, move || std::panic::panic_any(msg))
    }
}

mod wasi {
    use anyhow::Result;
    use wasmtime::{Config, Engine, Linker, Module, Store};
    use wasmtime_wasi::preview1::{self, WasiP1Ctx};
    use wasmtime_wasi::WasiCtxBuilder;

    fn run_wasi_test(wat: &'static str) -> Result<i32> {
        // Construct the wasm engine with async support disabled.
        let mut config = Config::new();
        config
            .async_support(false)
            .wasm_exceptions(true)
            .wasm_function_references(true)
            .wasm_typed_continuations(true);
        let engine = Engine::new(&config)?;

        // Add the WASI preview1 API to the linker (will be implemented in terms of
        // the preview2 API)
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        preview1::add_to_linker_sync(&mut linker, |t| t)?;

        // Add capabilities (e.g. filesystem access) to the WASI preview2 context here.
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();

        let mut store: Store<WasiP1Ctx> = Store::new(&engine, wasi_ctx);

        // Instantiate our wasm module.
        let module = Module::new(&engine, wat)?;
        let func = linker
            .module(&mut store, "", &module)?
            .get_default(&mut store, "")?
            .typed::<(), i32>(&store)?;

        // Invoke the WASI program default function.
        func.call(&mut store, ())
    }

    async fn run_wasi_test_async(wat: &'static str) -> Result<i32> {
        // Construct the wasm engine with async support enabled.
        let mut config = Config::new();
        config
            .async_support(true)
            .wasm_exceptions(true)
            .wasm_function_references(true)
            .wasm_typed_continuations(true);
        let engine = Engine::new(&config)?;

        // Add the WASI preview1 API to the linker (will be implemented in terms of
        // the preview2 API)
        let mut linker: Linker<WasiP1Ctx> = Linker::new(&engine);
        preview1::add_to_linker_async(&mut linker, |t| t)?;

        // Add capabilities (e.g. filesystem access) to the WASI preview2 context here.
        let wasi_ctx = WasiCtxBuilder::new().inherit_stdio().build_p1();

        let mut store: Store<WasiP1Ctx> = Store::new(&engine, wasi_ctx);

        // Instantiate our wasm module.
        let module = Module::new(&engine, wat)?;
        let func = linker
            .module_async(&mut store, "", &module)
            .await?
            .get_default(&mut store, "")?
            .typed::<(), i32>(&store)?;

        // Invoke the WASI program default function.
        func.call_async(&mut store, ()).await
    }

    static WRITE_SOMETHING_WAT: &'static str = &r#"
(module
  (type $ft (func (result i32)))
  (type $ct (cont $ft))
  (import "wasi_snapshot_preview1" "fd_write"
     (func $print (param $fd i32)
                  (param $iovec i32)
                  (param $len i32)
                  (param $written i32) (result i32)))
  (memory 1)
  (export "memory" (memory 0))

  ;; 9 is the offset to write to
  (data (i32.const 9) "something\n")

  (func $f (result i32)
    (i32.const 0) ;; offset
    (i32.const 9) ;; value start of the string
    (i32.store)

    (i32.const 4)                ;; offset
    (i32.const 11)               ;; value, the length of the string
    (i32.store offset=0 align=2) ;; size_buf_len

    (i32.const 1)   ;; 1 for stdout
    (i32.const 0)   ;; 0 as we stored the beginning of __wasi_ciovec_t
    (i32.const 1)   ;;
    (i32.const 20)  ;; nwritten
    (call $print)
  )
  (elem declare func $f)

  (func (export "_start") (result i32)
    (ref.func $f)
    (cont.new $ct)
    (resume $ct)
  )
)"#;

    #[test]
    fn write_something_test() -> Result<()> {
        assert_eq!(run_wasi_test(WRITE_SOMETHING_WAT)?, 0);
        Ok(())
    }

    #[tokio::test]
    async fn write_something_test_async() -> Result<()> {
        assert_eq!(run_wasi_test_async(WRITE_SOMETHING_WAT).await?, 0);
        Ok(())
    }

    static SCHED_YIELD_WAT: &'static str = r#"
(module
  (type $ft (func (result i32)))
  (type $ct (cont $ft))
  (import "wasi_snapshot_preview1" "sched_yield"
     (func $sched_yield (result i32)))
  (memory 1)
  (export "memory" (memory 0))

  (func $g (result i32)
    (call $sched_yield))
  (elem declare func $g)

  (func (export "_start") (result i32)
    (cont.new $ct (ref.func $g))
    (resume $ct)
  )
)"#;

    #[test]
    fn sched_yield_test() -> Result<()> {
        assert_eq!(run_wasi_test(SCHED_YIELD_WAT)?, 0);
        Ok(())
    }

    #[tokio::test]
    async fn sched_yield_test_async() -> Result<()> {
        assert_eq!(run_wasi_test_async(SCHED_YIELD_WAT).await?, 0);
        Ok(())
    }
}

/// Test that we can handle a `suspend` from another instance. Note that this
/// test is working around the fact that wasmtime does not support exporting
/// tags at the moment. Thus, instead of sharing a tag between two modules, we
/// instantiate the same module twice to share a tag.
#[test]
fn inter_instance_suspend() -> Result<()> {
    let mut config = Config::default();
    config.wasm_function_references(true);
    config.wasm_exceptions(true);
    config.wasm_typed_continuations(true);

    let engine = Engine::new(&config)?;

    let mut store = Store::<()>::new(&engine, ());

    let wat_other = r#"
        (module

          (type $ft (func))
          (type $ct (cont $ft))
          (tag $tag)


          (func $suspend (export "suspend")
            (suspend $tag)
          )

          (func $resume (export "resume") (param $f (ref $ct))
            (block $handler (result (ref $ct))
              (resume $ct (on $tag $handler) (local.get $f))
              (unreachable)
            )
            (drop)
          )
        )
    "#;

    let wat_main = r#"
        (module

          (type $ft (func))
          (type $ct (cont $ft))

          (import "other" "suspend" (func $suspend))
          (import "other" "resume" (func $resume (param (ref $ct))))

          (elem declare func $suspend)


          (func $entry (export "entry")
            (call $resume (cont.new $ct (ref.func $suspend)))
          )
        )
    "#;

    let module_other = Module::new(&engine, wat_other)?;

    let other_inst1 = Instance::new(&mut store, &module_other, &[])?;
    let other_inst2 = Instance::new(&mut store, &module_other, &[])?;

    // Crucially, suspend and resume are from two instances of the same module.
    let suspend = other_inst1.get_func(&mut store, "suspend").unwrap();
    let resume = other_inst2.get_func(&mut store, "resume").unwrap();

    let module_main = Module::new(&engine, wat_main)?;
    let main_instance = Instance::new(&mut store, &module_main, &[suspend.into(), resume.into()])?;
    let entry_func = main_instance.get_func(&mut store, "entry").unwrap();

    entry_func.call(&mut store, &[], &mut [])?;

    Ok(())
}

/// Tests interaction with host functions. Note that the interaction with host
/// functions and traps is covered by the module `traps` further down.
mod host {
    use super::test_utils::*;
    use wasmtime::*;

    const RE_ENTER_ON_CONTINUATION_ERROR: &'static str =
        "Re-entering wasm while already executing on a continuation stack";

    #[test]
    /// Tests calling a host function from within a wasm function running inside a continuation.
    /// Call chain:
    /// $entry -resume-> a -call-> host_func_a
    fn call_host_from_continuation() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (result i32)))
            (type $ct (cont $ft))

            (import "" "" (func $host_func_a (param i32) (result i32)))

            (func $a (export "a") (result i32)
                (call $host_func_a (i32.const 122))
            )
            (func $entry (export "entry") (result i32)
                (resume $ct (cont.new $ct (ref.func $a)))
            )
        )
    "#;

        let mut runner = Runner::new();

        let host_func_a = make_i32_inc_host_func(&mut runner);

        let result = runner.run_test::<i32>(wat, &[host_func_a.into()]).unwrap();
        assert_eq!(result, 123);
        Ok(())
    }

    #[test]
    /// We re-enter wasm from a host function and execute a continuation.
    /// Call chain:
    /// $entry -call-> $a -call-> $host_func_a -call-> $b -resume-> $c
    fn re_enter_wasm_ok1() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (param i32) (result i32)))
            (type $ct (cont $ft))

            (import "" "" (func $host_func_a (param i32) (result i32)))


            (func $a (export "a") (param $x i32) (result i32)
                (call $host_func_a (local.get $x))
            )

            (func $b (export "b") (param $x i32) (result i32)
                (resume $ct (local.get $x) (cont.new $ct (ref.func $c)))
            )

            (func $c (export "c") (param $x i32) (result i32)
                (return (i32.add (local.get $x) (i32.const 1)))
            )


            (func $entry (export "entry") (result i32)
                (call $a (i32.const 120))
            )
        )
    "#;

        let mut runner = Runner::new();

        let host_func_a = make_i32_inc_via_export_host_func(&mut runner, "b");

        let result = runner.run_test::<i32>(wat, &[host_func_a.into()]).unwrap();
        assert_eq!(result, 123);
        Ok(())
    }

    #[test]
    /// Similar to `re_enter_wasm_ok2, but we run a continuation before the host call.
    /// Call chain:
    /// $entry -call-> $a -call-> $host_func_a -call-> $b -resume-> $c
    fn re_enter_wasm_ok2() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (param i32) (result i32)))
            (type $ct (cont $ft))

            (import "" "" (func $host_func_a (param i32) (result i32)))


            (func $a (export "a") (param $x i32) (result i32)
                ;; Running continuation before calling into host is fine
                (resume $ct (local.get $x) (cont.new $ct (ref.func $c)))
                (drop)

                (call $host_func_a (local.get $x))
            )

            (func $b (export "b") (param $x i32) (result i32)
                (resume $ct (local.get $x) (cont.new $ct (ref.func $c)))
            )

            (func $c (export "c") (param $x i32) (result i32)
                (return (i32.add (local.get $x) (i32.const 1)))
            )


            (func $entry (export "entry") (result i32)
                (call $a (i32.const 120))
            )
        )
    "#;

        let mut runner = Runner::new();

        let host_func_a = make_i32_inc_via_export_host_func(&mut runner, "b");

        let result = runner.run_test::<i32>(wat, &[host_func_a.into()]).unwrap();
        assert_eq!(result, 123);
        Ok(())
    }

    #[cfg_attr(feature = "wasmfx_baseline", ignore)]
    #[test]
    /// We re-enter wasm from a host function while we were already on a continuation stack.
    /// This is currently forbidden (see wasmfx/wasmfxtime#109), but may be
    /// allowed in the future.
    /// Call chain:
    /// $entry -resume-> $a -call-> $host_func_a -call-> $b
    fn re_enter_wasm_bad() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (param i32) (result i32)))
            (type $ct (cont $ft))

            (import "" "" (func $host_func_a (param i32) (result i32)))


            (func $a (export "a") (param $x i32) (result i32)
                (call $host_func_a (local.get $x))
            )


            (func $b (export "b") (param $x i32) (result i32)
                 (return (i32.add (local.get $x) (i32.const 1)))
            )


            (func $entry (export "entry")
                (resume $ct (i32.const 120) (cont.new $ct (ref.func $a)))
                (drop)
            )
        )
    "#;
        let mut runner = Runner::new();

        let host_func_a = make_i32_inc_via_export_host_func(&mut runner, "b");

        runner.run_test_expect_str_error(
            &wat,
            &[host_func_a.into()],
            RE_ENTER_ON_CONTINUATION_ERROR,
        );
        Ok(())
    }

    #[cfg_attr(feature = "wasmfx_baseline", ignore)]
    #[test]
    /// After crossing from the host back into wasm, we suspend to a tag that is
    /// handled by the surrounding function (i.e., without needing to cross the
    /// host frame to reach the handler).
    /// This is currently forbidden (see wasmfx/wasmfxtime#109), but could be
    /// allowed in the future.
    /// Call chain:
    /// $entry -resume-> $a -call-> $host_func_a -call-> $b -resume-> $c
    fn call_host_from_continuation_nested_suspend_ok() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (param i32) (result i32)))
            (type $ct (cont $ft))
            (tag $t (result i32))

            (import "" "" (func $host_func_a (param i32) (result i32)))


            (func $a (export "a") (param $x i32) (result i32)
                (call $host_func_a (local.get $x))
            )


            (func $b (export "b") (param $x i32) (result i32)
                (block $h (result (ref $ct))
                  (resume $ct (on $t $h) (local.get $x) (cont.new $ct (ref.func $c)))
                  (unreachable)
                )
                (drop)
                ;; note that we do not run the continuation to completion
                (i32.add (local.get $x) (i32.const 1))
            )

            (func $c (export "c") (param $x i32) (result i32)
                (suspend $t)
            )


            (func $entry (export "entry")
                (resume $ct (i32.const 120) (cont.new $ct (ref.func $a)))
                (drop)
            )
        )
    "#;

        let mut runner = Runner::new();

        let host_func_a = make_i32_inc_via_export_host_func(&mut runner, "b");

        runner.run_test_expect_str_error(
            &wat,
            &[host_func_a.into()],
            RE_ENTER_ON_CONTINUATION_ERROR,
        );
        Ok(())
    }

    #[cfg_attr(feature = "wasmfx_baseline", ignore)]
    #[test]
    /// Similar to `call_host_from_continuation_nested_suspend_ok`. However,
    /// we suspend to a tag that is only handled if we were to cross a host function
    /// boundary.
    /// This currently triggers the check that we must not re-enter wasm while
    /// on a continuation (see wasmfx/wasmfxtime#109), but will most likely stay
    /// forbidden if host calls acts as barriers for suspensions. In that case,
    /// the test case will exhibit a case of suspending to an unhandled tag.
    ///
    /// Call chain:
    /// $entry -resume-> $a -call-> $host_func_a -call-> $b -resume-> $c
    fn call_host_from_continuation_nested_suspend_unhandled() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (param i32) (result i32)))
            (type $ct (cont $ft))
            (tag $t (result i32))

            (import "" "" (func $host_func_a (param i32) (result i32)))


            (func $a (export "a") (param $x i32) (result i32)
                (call $host_func_a (local.get $x))
            )


            (func $b (export "b") (param $x i32) (result i32)
                (resume $ct (local.get $x) (cont.new $ct (ref.func $c)))
            )

            (func $c (export "c") (param $x i32) (result i32)
                (suspend $t)
            )


            (func $entry (export "entry")
                (block $h (result (ref $ct))
                    (return
                        (resume $ct
                            (on $t $h)
                            (i32.const 123)
                            (cont.new $ct (ref.func $a))))
                )
                (drop)
            )
        )
    "#;

        let mut runner = Runner::new();

        let host_func_a = make_i32_inc_via_export_host_func(&mut runner, "b");

        runner.run_test_expect_str_error(
            &wat,
            &[host_func_a.into()],
            RE_ENTER_ON_CONTINUATION_ERROR,
        );
        Ok(())
    }
}

mod traps {
    use super::test_utils::*;
    use wasmtime::*;

    /// Runs the module given as `wat`. We expect execution to cause the
    /// `expected_trap` and a backtrace containing exactly the function names
    /// given by `expected_backtrace`.
    fn run_test_expect_trap_backtrace(wat: &str, expected_trap: Trap, expected_backtrace: &[&str]) {
        let runner = Runner::new();
        let result = runner.run_test::<()>(wat, &[]);

        let err = result.expect_err("Was expecting wasm execution to yield error");

        assert!(err.root_cause().is::<Trap>());
        assert_eq!(*err.downcast_ref::<Trap>().unwrap(), expected_trap);

        // In the baseline implementation, the stack trace will always be empty
        if !cfg!(feature = "wasmfx_baseline") {
            let trace = err.downcast_ref::<WasmBacktrace>().unwrap();

            let actual_func_name_it = trace
                .frames()
                .iter()
                .map(|frame| {
                    frame
                        .func_name()
                        .expect("Expecting all functions in actual backtrace to have names")
                })
                .rev();

            let expected_func_name_it = expected_backtrace.iter().map(|name| *name);

            assert!(actual_func_name_it.eq(expected_func_name_it));
        }
    }

    #[test]
    /// Tests that we get correct backtraces if we trap deep inside multiple continuations.
    /// Call chain:
    /// $entry -call-> $a -resume-> $b -call-> $c -resume-> $d -call-> $e -resume-> $f
    /// Here, $f traps.
    fn trap_in_continuation_nested() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
            )

            (func $b (export "b")
                (call $c)
            )

            (func $c (export "c")
                (resume $ct (cont.new $ct (ref.func $d)))
            )

            (func $d (export "d")
              (call $e)
            )

            (func $e (export "e")
                (resume $ct (cont.new $ct (ref.func $f)))
            )

            (func $f (export "f")
                (unreachable)
            )
        )
        "#;

        run_test_expect_trap_backtrace(
            wat,
            Trap::UnreachableCodeReached,
            &["entry", "a", "b", "c", "d", "e", "f"],
        );

        Ok(())
    }

    #[test]
    /// Tests that we get correct backtraces if we trap after returning from one
    /// continuation to its parent.
    fn trap_in_continuation_back_to_parent() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
            )

            (func $b (export "b")
                (call $c)
            )

            (func $c (export "c")
                (resume $ct (cont.new $ct (ref.func $d)))
                (unreachable)
            )

            (func $d (export "d")
                (call $e)
            )

            (func $e (export "e"))

        )
        "#;

        run_test_expect_trap_backtrace(
            wat,
            Trap::UnreachableCodeReached,
            &["entry", "a", "b", "c"],
        );

        Ok(())
    }

    #[test]
    /// Tests that we get correct backtraces if we trap after returning from
    /// several continuations back to the main stack.
    fn trap_in_continuation_back_to_main() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
                (unreachable)
            )

            (func $b (export "b")
                (call $c)
            )

            (func $c (export "c")
                (resume $ct (cont.new $ct (ref.func $d)))
            )

            (func $d (export "d")
              (call $e)
            )

            (func $e (export "e"))

        )
        "#;

        run_test_expect_trap_backtrace(wat, Trap::UnreachableCodeReached, &["entry", "a"]);

        Ok(())
    }

    #[test]
    /// Tests that we get correct backtraces after suspending a continuation.
    fn trap_in_continuation_suspend() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))

            (tag $t)

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
                (unreachable)
            )

            (func $b (export "b")
                (call $c)
            )

            (func $c (export "c")
                (block $handler (result  (ref $ct))
                    (resume $ct (on $t $handler) (cont.new $ct (ref.func $d)))
                    (return)
                )
                (unreachable)
            )

            (func $d (export "d")
                (call $e)
            )

            (func $e (export "e")
                (suspend $t)
            )

        )
    "#;

        run_test_expect_trap_backtrace(
            wat,
            Trap::UnreachableCodeReached,
            &["entry", "a", "b", "c"],
        );

        Ok(())
    }

    #[test]
    /// Tests that we get correct backtraces after suspending a continuation and
    /// then resuming it from a different stack frame.
    fn trap_in_continuation_suspend_resume() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))

            (tag $t)

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
            )

            (func $b (export "b")
                (resume $ct (call $c))
            )

            (func $c (export "c") (result (ref $ct))
                (block $handler (result  (ref $ct))
                    (resume $ct (on $t $handler) (cont.new $ct (ref.func $d)))

                    ;; We never want to get here, but also don't want to use
                    ;; (unreachable), which is the trap we deliberately use in
                    ;; this test. Instead, we call a null function ref here,
                    ;; which is guaranteed to trap.
                    (call_ref $ft (ref.null $ft))

                    (return (cont.new $ct (ref.func $d)))
                )
                ;; implicitly returning the continuation here
            )

            (func $d (export "d")
                (call $e)
                (unreachable)
            )

            (func $e (export "e")
                (suspend $t)
            )

        )
    "#;

        // Note that c does not appear in the stack trace:
        // In $b, we resume the suspended computation, which started in $d,
        // suspended in $e, and traps in $d
        run_test_expect_trap_backtrace(
            wat,
            Trap::UnreachableCodeReached,
            &["entry", "a", "b", "d"],
        );

        Ok(())
    }

    #[test]
    /// Tests that we get correct backtraces after suspending a continuation
    /// where we need to forward to an outer handler.
    fn trap_in_continuation_forward() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))
            (tag $t)

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (block $handler (result  (ref $ct))
                    (resume $ct (on $t $handler) (cont.new $ct (ref.func $b)))
                    ;; We don't actually want to get here
                    (return)
                )
                (unreachable)
            )

            (func $b (export "b")
                (call $c)
            )

            (func $c (export "c")
                (resume $ct (cont.new $ct (ref.func $d)))
            )

            (func $d (export "d")
                (call $e)
            )

            (func $e (export "e")
                (suspend $t)
            )

        )
    "#;

        // Note that c does not appear in the stack trace:
        // In $b, we resume the suspended computation, which started in $d,
        // suspended in $e, and traps in $d
        run_test_expect_trap_backtrace(wat, Trap::UnreachableCodeReached, &["entry", "a"]);

        Ok(())
    }

    #[test]
    /// Tests that we get correct backtraces after suspending a continuation
    /// where we need to forward to an outer handler. We then resume the
    /// continuation from within another continuation.
    fn trap_in_continuation_forward_resume() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))
            (tag $t)

            (global $k (mut (ref null $ct)) (ref.null $ct))

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
            )

            (func $b (export "b")
                (block $handler (result  (ref $ct))
                    (resume $ct (on $t $handler) (cont.new $ct (ref.func $c)))
                    ;; We don't actually want to get here
                    (return)
                )
                (global.set $k)
                ;; $f will resume $k
                (resume $ct (cont.new $ct (ref.func $f)))
            )

            (func $c (export "c")
                (resume $ct (cont.new $ct (ref.func $d)))
            )

            (func $d (export "d")
                (call $e)
            )

            (func $e (export "e")
                (suspend $t)
                (unreachable)
            )

           (func $f  (export "f")
               (resume $ct (global.get $k))
           )
        )
       "#;

        run_test_expect_trap_backtrace(
            wat,
            Trap::UnreachableCodeReached,
            &["entry", "a", "b", "f", "c", "d", "e"],
        );

        Ok(())
    }

    #[test]
    /// Tests that we get correct panic payloads  if we panic deep inside multiple
    /// continuations. Note that wasmtime does not create its own backtraces for panics.
    fn panic_in_continuation() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func))
            (type $ct (cont $ft))

            (import "" "" (func $f))

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                (resume $ct (cont.new $ct (ref.func $b)))
            )

            (func $b (export "b")
                (call $c)
            )

            (func $c (export "c")
                (resume $ct (cont.new $ct (ref.func $d)))
            )

            (func $d (export "d")
                (call $e)
            )

            (func $e (export "e")
                (call $f)
            )

        )
        "#;

        let mut runner = Runner::new();

        let msg = "Host function f panics";

        let f = make_panicking_host_func(&mut runner.store, msg);
        let error = runner.run_test_expect_panic(wat, &[f.into()]);
        assert_eq!(error.downcast_ref::<&'static str>(), Some(&msg));

        Ok(())
    }

    #[test]
    #[cfg_attr(feature = "wasmfx_baseline", ignore)]
    fn stack_overflow_in_continuation() -> Result<()> {
        let wat = r#"
        (module
            (type $ft (func (param i32)))
            (type $ct (cont $ft))

            (func $entry (export "entry")
                (call $a)
            )

            (func $a (export "a")
                ;; We ask for a billion recursive calls
                (i32.const 1_000_000_000)

                (resume $ct (cont.new $ct (ref.func $overflow)))
            )

            (func $overflow (export "overflow") (param $i i32)
                (block $continue
                    (local.get $i)
                    ;; return if $i == 0
                    (br_if $continue)
                    (return)
                )
                (i32.sub (local.get $i) (i32.const 1))
                (call $overflow)
            )

        )
    "#;

        let runner = Runner::new();

        let error = runner
            .run_test::<()>(wat, &[])
            .expect_err("Expecting execution to yield error");

        assert!(error.root_cause().is::<Trap>());
        assert_eq!(*error.downcast_ref::<Trap>().unwrap(), Trap::StackOverflow);

        Ok(())
    }
}

mod misc {
    use super::test_utils::*;
    use wasmtime::*;

    #[ignore]
    #[test]
    pub fn continuation_revision_counter_wraparound() -> Result<()> {
        let wat = r#"
(module
  (type $ft (func))
  (type $ct (cont $ft))

  (tag $yield)

  (func $loop
    (loop $loop
      (suspend $yield)
      (br $loop)
    )
  )
  (elem declare func $loop)

  ;; Loops 65536 times to overflow the 16 bit revision counter on the continuation reference.
  (func (export "entry")
    (local $k (ref $ct))
    (local $i i32)
    (local.set $k (cont.new $ct (ref.func $loop)))
    (loop $go-again
      (block $on-yield (result (ref $ct))
        (resume $ct (on $yield $on-yield) (local.get $k))
        (unreachable)
      )
      (local.set $k)
      (local.set $i (i32.add (i32.const 1) (local.get $i)))
      (br_if $go-again (i32.lt_u (local.get $i) (i32.const 65536)))
    )
  )
)
"#;

        let runner = Runner::new();
        cfg_if::cfg_if! {
            if #[cfg(feature = "unsafe_disable_continuation_linearity_check")] {
                let result = runner.run_test::<()>(wat, &[])?;
                assert_eq!(result, ())
            } else {
                let error = runner.run_test::<()>(wat, &[])
                    .expect_err("expected an overflow");
                assert!(error.root_cause().is::<Trap>());
                assert_eq!(*error.downcast_ref::<Trap>().unwrap(), Trap::IntegerOverflow);
            }
        }
        Ok(())
    }
}
