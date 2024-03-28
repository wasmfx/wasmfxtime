// Generate traits for synchronous bindings.
//
// Note that this is only done for interfaces which can block, or those which
// have some functions in `only_imports` below for being async.
pub mod sync {
    mod generated {
        use crate::{FsError, StreamError};

        wasmtime::component::bindgen!({
            path: "wit",
            world: "wasi:cli/command",
            tracing: true,
            trappable_error_type: {
                "wasi:io/streams/stream-error" => StreamError,
                "wasi:filesystem/types/error-code" => FsError,
            },
            with: {
                // These interfaces come from the outer module, as it's
                // sync/async agnostic.
                "wasi:clocks": crate::bindings::clocks,
                "wasi:random": crate::bindings::random,
                "wasi:sockets": crate::bindings::sockets,
                "wasi:cli": crate::bindings::cli,
                "wasi:io/error": crate::bindings::io::error,
                "wasi:filesystem/preopens": crate::bindings::filesystem::preopens,

                // Configure the resource types of the bound interfaces here
                // to be the same as the async versions of the resources, that
                // way everything has the same type.
                "wasi:filesystem/types/descriptor": super::super::filesystem::types::Descriptor,
                "wasi:filesystem/types/directory-entry-stream": super::super::filesystem::types::DirectoryEntryStream,
                "wasi:io/poll/pollable": super::super::io::poll::Pollable,
                "wasi:io/streams/input-stream": super::super::io::streams::InputStream,
                "wasi:io/streams/output-stream": super::super::io::streams::OutputStream,
            }
        });
    }
    pub use self::generated::exports;
    pub use self::generated::wasi::{filesystem, io};

    /// Synchronous bindings to execute and run a `wasi:cli/command`.
    ///
    /// This structure is automatically generated by `bindgen!` and is intended
    /// to be used with [`Config::async_support(false)`][async]. For the
    /// asynchronous version see [`bindings::Command`](super::Command).
    ///
    /// This can be used for a more "typed" view of executing a command
    /// component through the [`Command::wasi_cli_run`] method plus
    /// [`Guest::call_run`](exports::wasi::cli::run::Guest::call_run).
    ///
    /// > **Note**: it's recommended to use
    /// > [`wasmtime_wasi::add_to_linker_sync`] instead of the auto-generated
    /// > [`Command::add_to_linker`] here.
    ///
    /// [async]: wasmtime::Config::async_support
    /// [`wasmtime_wasi::add_to_linker_sync`]: crate::add_to_linker_sync
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use wasmtime::{Engine, Result, Store, Config};
    /// use wasmtime::component::{ResourceTable, Linker, Component};
    /// use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder};
    /// use wasmtime_wasi::bindings::sync::Command;
    ///
    /// // This example is an example shim of executing a component based on the
    /// // command line arguments provided to this program.
    /// fn main() -> Result<()> {
    ///     let args = std::env::args().skip(1).collect::<Vec<_>>();
    ///
    ///     // Configure and create `Engine`
    ///     let engine = Engine::default();
    ///
    ///     // Configure a `Linker` with WASI, compile a component based on
    ///     // command line arguments, and then pre-instantiate it.
    ///     let mut linker = Linker::<MyState>::new(&engine);
    ///     wasmtime_wasi::add_to_linker_async(&mut linker)?;
    ///     let component = Component::from_file(&engine, &args[0])?;
    ///     let pre = linker.instantiate_pre(&component)?;
    ///
    ///
    ///     // Configure a `WasiCtx` based on this program's environment. Then
    ///     // build a `Store` to instantiate into.
    ///     let mut builder = WasiCtxBuilder::new();
    ///     builder.inherit_stdio().inherit_env().args(&args);
    ///     let mut store = Store::new(
    ///         &engine,
    ///         MyState {
    ///             ctx: builder.build(),
    ///             table: ResourceTable::new(),
    ///         },
    ///     );
    ///
    ///     // Instantiate the component and we're off to the races.
    ///     let (command, _instance) = Command::instantiate_pre(&mut store, &pre)?;
    ///     let program_result = command.wasi_cli_run().call_run(&mut store)?;
    ///     match program_result {
    ///         Ok(()) => Ok(()),
    ///         Err(()) => std::process::exit(1),
    ///     }
    /// }
    ///
    /// struct MyState {
    ///     ctx: WasiCtx,
    ///     table: ResourceTable,
    /// }
    ///
    /// impl WasiView for MyState {
    ///     fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
    ///     fn table(&mut self) -> &mut ResourceTable { &mut self.table }
    /// }
    /// ```
    pub use self::generated::Command;
}

mod async_io {
    wasmtime::component::bindgen!({
        path: "wit",
        world: "wasi:cli/command",
        tracing: true,
        async: {
            // Only these functions are `async` and everything else is sync
            // meaning that it basically doesn't need to block. These functions
            // are the only ones that need to block.
            //
            // Note that at this time `only_imports` works on function names
            // which in theory can be shared across interfaces, so this may
            // need fancier syntax in the future.
            only_imports: [
                "[method]descriptor.access-at",
                "[method]descriptor.advise",
                "[method]descriptor.change-directory-permissions-at",
                "[method]descriptor.change-file-permissions-at",
                "[method]descriptor.create-directory-at",
                "[method]descriptor.get-flags",
                "[method]descriptor.get-type",
                "[method]descriptor.is-same-object",
                "[method]descriptor.link-at",
                "[method]descriptor.lock-exclusive",
                "[method]descriptor.lock-shared",
                "[method]descriptor.metadata-hash",
                "[method]descriptor.metadata-hash-at",
                "[method]descriptor.open-at",
                "[method]descriptor.read",
                "[method]descriptor.read-directory",
                "[method]descriptor.readlink-at",
                "[method]descriptor.remove-directory-at",
                "[method]descriptor.rename-at",
                "[method]descriptor.set-size",
                "[method]descriptor.set-times",
                "[method]descriptor.set-times-at",
                "[method]descriptor.stat",
                "[method]descriptor.stat-at",
                "[method]descriptor.symlink-at",
                "[method]descriptor.sync",
                "[method]descriptor.sync-data",
                "[method]descriptor.try-lock-exclusive",
                "[method]descriptor.try-lock-shared",
                "[method]descriptor.unlink-file-at",
                "[method]descriptor.unlock",
                "[method]descriptor.write",
                "[method]input-stream.read",
                "[method]input-stream.blocking-read",
                "[method]input-stream.blocking-skip",
                "[method]input-stream.skip",
                "[method]output-stream.forward",
                "[method]output-stream.splice",
                "[method]output-stream.blocking-splice",
                "[method]output-stream.blocking-flush",
                "[method]output-stream.blocking-write",
                "[method]output-stream.blocking-write-and-flush",
                "[method]output-stream.blocking-write-zeroes-and-flush",
                "[method]directory-entry-stream.read-directory-entry",
                "poll",
                "[method]pollable.block",
                "[method]pollable.ready",
            ],
        },
        trappable_error_type: {
            "wasi:io/streams/stream-error" => crate::StreamError,
            "wasi:filesystem/types/error-code" => crate::FsError,
            "wasi:sockets/network/error-code" => crate::SocketError,
        },
        with: {
            // Configure all resources to be concrete types defined in this crate,
            // so that way we get to use nice typed helper methods with
            // `ResourceTable`.
            "wasi:sockets/network/network": crate::network::Network,
            "wasi:sockets/tcp/tcp-socket": crate::tcp::TcpSocket,
            "wasi:sockets/udp/udp-socket": crate::udp::UdpSocket,
            "wasi:sockets/udp/incoming-datagram-stream": crate::udp::IncomingDatagramStream,
            "wasi:sockets/udp/outgoing-datagram-stream": crate::udp::OutgoingDatagramStream,
            "wasi:sockets/ip-name-lookup/resolve-address-stream": crate::ip_name_lookup::ResolveAddressStream,
            "wasi:filesystem/types/directory-entry-stream": crate::filesystem::ReaddirIterator,
            "wasi:filesystem/types/descriptor": crate::filesystem::Descriptor,
            "wasi:io/streams/input-stream": crate::stream::InputStream,
            "wasi:io/streams/output-stream": crate::stream::OutputStream,
            "wasi:io/error/error": crate::stream::Error,
            "wasi:io/poll/pollable": crate::poll::Pollable,
            "wasi:cli/terminal-input/terminal-input": crate::stdio::TerminalInput,
            "wasi:cli/terminal-output/terminal-output": crate::stdio::TerminalOutput,
        },
    });
}

pub use self::async_io::exports;
pub use self::async_io::wasi::*;

/// Asynchronous bindings to execute and run a `wasi:cli/command`.
///
/// This structure is automatically generated by `bindgen!` and is intended to
/// be used with [`Config::async_support(true)`][async]. For the synchronous
/// version see [`binidngs::sync::Command`](sync::Command).
///
/// This can be used for a more "typed" view of executing a command component
/// through the [`Command::wasi_cli_run`] method plus
/// [`Guest::call_run`](exports::wasi::cli::run::Guest::call_run).
///
/// > **Note**: it's recommended to use [`wasmtime_wasi::add_to_linker_async`]
/// > instead of the auto-generated [`Command::add_to_linker`] here.
///
/// [async]: wasmtime::Config::async_support
/// [`wasmtime_wasi::add_to_linker_async`]: crate::add_to_linker_async
///
/// # Examples
///
/// ```no_run
/// use wasmtime::{Engine, Result, Store, Config};
/// use wasmtime::component::{ResourceTable, Linker, Component};
/// use wasmtime_wasi::{WasiCtx, WasiView, WasiCtxBuilder};
/// use wasmtime_wasi::bindings::Command;
///
/// // This example is an example shim of executing a component based on the
/// // command line arguments provided to this program.
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     let args = std::env::args().skip(1).collect::<Vec<_>>();
///
///     // Configure and create `Engine`
///     let mut config = Config::new();
///     config.async_support(true);
///     let engine = Engine::new(&config)?;
///
///     // Configure a `Linker` with WASI, compile a component based on
///     // command line arguments, and then pre-instantiate it.
///     let mut linker = Linker::<MyState>::new(&engine);
///     wasmtime_wasi::add_to_linker_async(&mut linker)?;
///     let component = Component::from_file(&engine, &args[0])?;
///     let pre = linker.instantiate_pre(&component)?;
///
///
///     // Configure a `WasiCtx` based on this program's environment. Then
///     // build a `Store` to instantiate into.
///     let mut builder = WasiCtxBuilder::new();
///     builder.inherit_stdio().inherit_env().args(&args);
///     let mut store = Store::new(
///         &engine,
///         MyState {
///             ctx: builder.build(),
///             table: ResourceTable::new(),
///         },
///     );
///
///     // Instantiate the component and we're off to the races.
///     let (command, _instance) = Command::instantiate_pre(&mut store, &pre).await?;
///     let program_result = command.wasi_cli_run().call_run(&mut store).await?;
///     match program_result {
///         Ok(()) => Ok(()),
///         Err(()) => std::process::exit(1),
///     }
/// }
///
/// struct MyState {
///     ctx: WasiCtx,
///     table: ResourceTable,
/// }
///
/// impl WasiView for MyState {
///     fn ctx(&mut self) -> &mut WasiCtx { &mut self.ctx }
///     fn table(&mut self) -> &mut ResourceTable { &mut self.table }
/// }
/// ```
pub use self::async_io::Command;
