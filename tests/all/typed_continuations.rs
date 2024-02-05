use anyhow::Result;
use wasmtime::*;
use wasmtime_wasi::preview2;

struct WasiHostCtx {
    preview2_ctx: preview2::WasiCtx,
    preview2_table: wasmtime::component::ResourceTable,
    preview1_adapter: preview2::preview1::WasiPreview1Adapter,
}

impl preview2::WasiView for WasiHostCtx {
    fn table(&mut self) -> &mut wasmtime::component::ResourceTable {
        &mut self.preview2_table
    }

    fn ctx(&mut self) -> &mut preview2::WasiCtx {
        &mut self.preview2_ctx
    }
}

impl preview2::preview1::WasiPreview1View for WasiHostCtx {
    fn adapter(&self) -> &preview2::preview1::WasiPreview1Adapter {
        &self.preview1_adapter
    }

    fn adapter_mut(&mut self) -> &mut preview2::preview1::WasiPreview1Adapter {
        &mut self.preview1_adapter
    }
}

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
    let mut linker: Linker<WasiHostCtx> = Linker::new(&engine);
    preview2::preview1::add_to_linker_sync(&mut linker)?;

    // Add capabilities (e.g. filesystem access) to the WASI preview2 context here.
    let wasi_ctx = preview2::WasiCtxBuilder::new().inherit_stdio().build();

    let host_ctx = WasiHostCtx {
        preview2_ctx: wasi_ctx,
        preview2_table: preview2::ResourceTable::new(),
        preview1_adapter: preview2::preview1::WasiPreview1Adapter::new(),
    };
    let mut store: Store<WasiHostCtx> = Store::new(&engine, host_ctx);

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
    let mut linker: Linker<WasiHostCtx> = Linker::new(&engine);
    preview2::preview1::add_to_linker_async(&mut linker)?;

    // Add capabilities (e.g. filesystem access) to the WASI preview2 context here.
    let wasi_ctx = preview2::WasiCtxBuilder::new().inherit_stdio().build();

    let host_ctx = WasiHostCtx {
        preview2_ctx: wasi_ctx,
        preview2_table: preview2::ResourceTable::new(),
        preview1_adapter: preview2::preview1::WasiPreview1Adapter::new(),
    };
    let mut store: Store<WasiHostCtx> = Store::new(&engine, host_ctx);

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
