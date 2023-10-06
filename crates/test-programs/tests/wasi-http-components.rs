#![cfg(all(feature = "test_programs", not(skip_wasi_http_tests)))]

use anyhow::Result;
use test_programs::http_server::Server;
use wasmtime::{
    component::{Component, Linker},
    Config, Engine, Store,
};
use wasmtime_wasi::preview2::{
    command::Command, pipe::MemoryOutputPipe, Table, WasiCtx, WasiCtxBuilder, WasiView,
};
use wasmtime_wasi_http::{WasiHttpCtx, WasiHttpView};

lazy_static::lazy_static! {
    static ref ENGINE: Engine = {
        let mut config = Config::new();
        config.wasm_backtrace_details(wasmtime::WasmBacktraceDetails::Enable);
        config.wasm_component_model(true);
        config.async_support(true);
        let engine = Engine::new(&config).unwrap();
        engine
    };
}
// uses ENGINE, creates a fn get_module(&str) -> Module
include!(concat!(env!("OUT_DIR"), "/wasi_http_tests_components.rs"));

struct Ctx {
    table: Table,
    wasi: WasiCtx,
    http: WasiHttpCtx,
}

impl WasiView for Ctx {
    fn table(&self) -> &Table {
        &self.table
    }
    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }
    fn ctx(&self) -> &WasiCtx {
        &self.wasi
    }
    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.wasi
    }
}

impl WasiHttpView for Ctx {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

async fn instantiate_component(
    component: Component,
    ctx: Ctx,
) -> Result<(Store<Ctx>, Command), anyhow::Error> {
    let mut linker = Linker::new(&ENGINE);
    wasmtime_wasi_http::proxy::add_to_linker(&mut linker)?;

    let mut store = Store::new(&ENGINE, ctx);

    let (command, _instance) = Command::instantiate_async(&mut store, &component, &linker).await?;
    Ok((store, command))
}

async fn run(name: &str, server: &Server) -> Result<()> {
    let stdout = MemoryOutputPipe::new(4096);
    let stderr = MemoryOutputPipe::new(4096);
    let r = {
        let table = Table::new();
        let component = get_component(name);

        // Create our wasi context.
        let mut builder = WasiCtxBuilder::new();
        builder.stdout(stdout.clone());
        builder.stderr(stderr.clone());
        builder.arg(name);
        for (var, val) in test_programs::wasi_tests_environment() {
            builder.env(var, val);
        }
        builder.env("HTTP_SERVER", server.addr());
        let wasi = builder.build();
        let http = WasiHttpCtx;

        let (mut store, command) =
            instantiate_component(component, Ctx { table, wasi, http }).await?;
        command.wasi_cli_run().call_run(&mut store).await
    };
    r.map_err(move |trap: anyhow::Error| {
        let stdout = stdout.try_into_inner().expect("single ref to stdout");
        if !stdout.is_empty() {
            println!("[guest] stdout:\n{}\n===", String::from_utf8_lossy(&stdout));
        }
        let stderr = stderr.try_into_inner().expect("single ref to stderr");
        if !stderr.is_empty() {
            println!("[guest] stderr:\n{}\n===", String::from_utf8_lossy(&stderr));
        }
        trap.context(format!(
            "error while testing wasi-tests {} with http-components",
            name
        ))
    })?
    .map_err(|()| anyhow::anyhow!("run returned an error"))?;
    Ok(())
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_get() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_get", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_post() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_post", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_large_post() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_large_post", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_put() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_put", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_invalid_version() -> Result<()> {
    let server = Server::http2()?;
    run("outbound_request_invalid_version", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_unknown_method() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_unknown_method", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_unsupported_scheme() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_unsupported_scheme", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_invalid_port() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_invalid_port", &server).await
}

#[test_log::test(tokio::test(flavor = "multi_thread"))]
async fn outbound_request_invalid_dnsname() -> Result<()> {
    let server = Server::http1()?;
    run("outbound_request_invalid_dnsname", &server).await
}
