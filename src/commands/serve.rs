use crate::common::{Profile, RunCommon, RunTarget};
use anyhow::{bail, Result};
use clap::Parser;
use std::{
    path::PathBuf,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use wasmtime::component::{InstancePre, Linker};
use wasmtime::{Engine, Store, StoreLimits};
use wasmtime_wasi::preview2::{Table, WasiCtx, WasiCtxBuilder, WasiView};
use wasmtime_wasi_http::{body::HyperOutgoingBody, WasiHttpCtx, WasiHttpView};

#[cfg(feature = "wasi-nn")]
use wasmtime_wasi_nn::WasiNnCtx;

struct Host {
    table: Table,
    ctx: WasiCtx,
    http: WasiHttpCtx,

    limits: StoreLimits,

    #[cfg(feature = "wasi-nn")]
    nn: Option<WasiNnCtx>,
}

impl WasiView for Host {
    fn table(&self) -> &Table {
        &self.table
    }

    fn table_mut(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&self) -> &WasiCtx {
        &self.ctx
    }

    fn ctx_mut(&mut self) -> &mut WasiCtx {
        &mut self.ctx
    }
}

impl WasiHttpView for Host {
    fn table(&mut self) -> &mut Table {
        &mut self.table
    }

    fn ctx(&mut self) -> &mut WasiHttpCtx {
        &mut self.http
    }
}

const DEFAULT_ADDR: std::net::SocketAddr = std::net::SocketAddr::new(
    std::net::IpAddr::V4(std::net::Ipv4Addr::new(0, 0, 0, 0)),
    8080,
);

/// Runs a WebAssembly module
#[derive(Parser)]
#[structopt(name = "serve")]
pub struct ServeCommand {
    #[clap(flatten)]
    run: RunCommon,

    /// Socket address for the web server to bind to.
    #[clap(long = "addr", value_name = "SOCKADDR", default_value_t = DEFAULT_ADDR )]
    addr: std::net::SocketAddr,

    /// The WebAssembly component to run.
    #[clap(value_name = "WASM", required = true)]
    component: PathBuf,
}

impl ServeCommand {
    /// Start a server to run the given wasi-http proxy component
    pub fn execute(mut self) -> Result<()> {
        self.run.common.init_logging();

        // We force cli errors before starting to listen for connections so tha we don't
        // accidentally delay them to the first request.
        if self.run.common.wasi.nn == Some(true) {
            #[cfg(not(feature = "wasi-nn"))]
            {
                bail!("Cannot enable wasi-nn when the binary is not compiled with this feature.");
            }
        }

        if let Some(Profile::Guest { .. }) = &self.run.profile {
            bail!("Cannot use the guest profiler with components");
        }

        if self.run.common.wasi.nn == Some(true) {
            #[cfg(not(feature = "wasi-nn"))]
            {
                bail!("Cannot enable wasi-nn when the binary is not compiled with this feature.");
            }
        }

        if self.run.common.wasi.threads == Some(true) {
            bail!("wasi-threads does not support components yet")
        }

        // The serve command requires both wasi-http and the component model, so we enable those by
        // default here.
        if self.run.common.wasi.http.replace(true) == Some(false) {
            bail!("wasi-http is required for the serve command, and must not be disabled");
        }
        if self.run.common.wasm.component_model.replace(true) == Some(false) {
            bail!("components are required for the serve command, and must not be disabled");
        }

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_time()
            .enable_io()
            .build()?;

        runtime.block_on(async move {
            tokio::select! {
                _ = tokio::signal::ctrl_c() => {
                    Ok::<_, anyhow::Error>(())
                }

                res = self.serve() => {
                    res
                }
            }
        })?;

        Ok(())
    }

    fn new_store(&self, engine: &Engine) -> Result<Store<Host>> {
        let mut builder = WasiCtxBuilder::new();

        // TODO: connect stdio to logging infrastructure

        let mut host = Host {
            table: Table::new(),
            ctx: builder.build(),
            http: WasiHttpCtx,

            limits: StoreLimits::default(),

            #[cfg(feature = "wasi-nn")]
            nn: None,
        };

        if self.run.common.wasi.nn == Some(true) {
            #[cfg(feature = "wasi-nn")]
            {
                let graphs = self
                    .run
                    .common
                    .wasi
                    .nn_graph
                    .iter()
                    .map(|g| (g.format.clone(), g.dir.clone()))
                    .collect::<Vec<_>>();
                let (backends, registry) = wasmtime_wasi_nn::preload(&graphs)?;
                host.nn.replace(WasiNnCtx::new(backends, registry));
            }
        }

        let mut store = Store::new(engine, host);

        if self.run.common.wasm.timeout.is_some() {
            store.set_epoch_deadline(1);
        }

        store.data_mut().limits = self.run.store_limits();
        store.limiter(|t| &mut t.limits);

        // If fuel has been configured, we want to add the configured
        // fuel amount to this store.
        if let Some(fuel) = self.run.common.wasm.fuel {
            store.add_fuel(fuel)?;
        }

        Ok(store)
    }

    fn add_to_linker(&self, linker: &mut Linker<Host>) -> Result<()> {
        // wasi-http and the component model are implicitly enabled for `wasmtime serve`, so we
        // don't test for `self.run.common.wasi.common` or `self.run.common.wasi.http` in this
        // function.

        wasmtime_wasi_http::proxy::add_to_linker(linker)?;

        if self.run.common.wasi.nn == Some(true) {
            #[cfg(feature = "wasi-nn")]
            {
                wasmtime_wasi_nn::wit::ML::add_to_linker(linker, |host| host.nn.as_mut().unwrap())?;
            }
        }

        Ok(())
    }

    async fn serve(mut self) -> Result<()> {
        use hyper::server::conn::http1;

        let mut config = self.run.common.config(None)?;
        config.wasm_component_model(true);
        config.async_support(true);

        if self.run.common.wasm.timeout.is_some() {
            config.epoch_interruption(true);
        }

        match self.run.profile {
            Some(Profile::Native(s)) => {
                config.profiler(s);
            }

            // We bail early in `execute` if the guest profiler is configured.
            Some(Profile::Guest { .. }) => unreachable!(),

            None => {}
        }

        let engine = Engine::new(&config)?;
        let mut linker = Linker::new(&engine);

        self.add_to_linker(&mut linker)?;

        let component = match self.run.load_module(&engine, &self.component)? {
            RunTarget::Core(_) => bail!("The serve command currently requires a component"),
            RunTarget::Component(c) => c,
        };

        let instance = linker.instantiate_pre(&component)?;

        let listener = tokio::net::TcpListener::bind(self.addr).await?;

        let _epoch_thread = if let Some(timeout) = self.run.common.wasm.timeout {
            Some(EpochThread::spawn(timeout, engine.clone()))
        } else {
            None
        };

        let handler = ProxyHandler::new(self, engine, instance);

        loop {
            let (stream, _) = listener.accept().await?;
            let h = handler.clone();
            tokio::task::spawn(async move {
                if let Err(e) = http1::Builder::new()
                    .keep_alive(true)
                    .serve_connection(stream, h)
                    .await
                {
                    eprintln!("error: {e:?}");
                }
            });
        }
    }
}

struct EpochThread {
    shutdown: Arc<AtomicBool>,
    handle: Option<std::thread::JoinHandle<()>>,
}

impl EpochThread {
    fn spawn(timeout: std::time::Duration, engine: Engine) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let handle = {
            let shutdown = Arc::clone(&shutdown);
            let handle = std::thread::spawn(move || {
                while !shutdown.load(Ordering::Relaxed) {
                    std::thread::sleep(timeout);
                    engine.increment_epoch();
                }
            });
            Some(handle)
        };

        EpochThread { shutdown, handle }
    }
}

impl Drop for EpochThread {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            self.shutdown.store(true, Ordering::Relaxed);
            handle.join().unwrap();
        }
    }
}

struct ProxyHandlerInner {
    cmd: ServeCommand,
    engine: Engine,
    instance_pre: InstancePre<Host>,
}

#[derive(Clone)]
struct ProxyHandler(Arc<ProxyHandlerInner>);

impl ProxyHandler {
    fn new(cmd: ServeCommand, engine: Engine, instance_pre: InstancePre<Host>) -> Self {
        Self(Arc::new(ProxyHandlerInner {
            cmd,
            engine,
            instance_pre,
        }))
    }
}

type Request = hyper::Request<hyper::body::Incoming>;

impl hyper::service::Service<Request> for ProxyHandler {
    type Response = hyper::Response<HyperOutgoingBody>;
    type Error = anyhow::Error;
    type Future = Pin<Box<dyn std::future::Future<Output = Result<Self::Response>> + Send>>;

    fn call(&mut self, req: Request) -> Self::Future {
        use http_body_util::BodyExt;

        let handler = self.clone();

        let (sender, receiver) = tokio::sync::oneshot::channel();

        // TODO: need to track the join handle, but don't want to block the response on it
        tokio::task::spawn(async move {
            let mut store = handler.0.cmd.new_store(&handler.0.engine)?;

            let req = store.data_mut().new_incoming_request(
                req.map(|body| body.map_err(|e| anyhow::anyhow!(e)).boxed()),
            )?;

            let out = store.data_mut().new_response_outparam(sender)?;

            let (proxy, _inst) = wasmtime_wasi_http::proxy::Proxy::instantiate_pre(
                &mut store,
                &handler.0.instance_pre,
            )
            .await?;

            proxy
                .wasi_http_incoming_handler()
                .call_handle(store, req, out)
                .await?;

            Ok::<_, anyhow::Error>(())
        });

        Box::pin(async move {
            let resp = receiver.await.unwrap()?;
            Ok(resp)
        })
    }
}
