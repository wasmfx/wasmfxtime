use crate::preview2::bindings::{
    io::streams::{InputStream, OutputStream},
    sockets::network::{ErrorCode, IpAddressFamily, IpSocketAddress, Network},
    sockets::tcp::{self, ShutdownType},
};
use crate::preview2::tcp::{TcpSocket, TcpState};
use crate::preview2::{Pollable, SocketResult, WasiView};
use cap_net_ext::{Blocking, PoolExt, TcpListenerExt};
use cap_std::net::TcpListener;
use io_lifetimes::AsSocketlike;
use rustix::io::Errno;
use rustix::net::sockopt;
use tokio::io::Interest;
use wasmtime::component::Resource;

impl<T: WasiView> tcp::Host for T {}

impl<T: WasiView> crate::preview2::host::tcp::tcp::HostTcpSocket for T {
    fn start_bind(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        network: Resource<Network>,
        local_address: IpSocketAddress,
    ) -> SocketResult<()> {
        let table = self.table_mut();
        let socket = table.get_resource(&this)?;

        match socket.tcp_state {
            TcpState::Default => {}
            _ => return Err(ErrorCode::NotInProgress.into()),
        }

        let network = table.get_resource(&network)?;
        let binder = network.pool.tcp_binder(local_address)?;

        // Perform the OS bind call.
        binder.bind_existing_tcp_listener(
            &*socket.tcp_socket().as_socketlike_view::<TcpListener>(),
        )?;

        let socket = table.get_resource_mut(&this)?;
        socket.tcp_state = TcpState::BindStarted;

        Ok(())
    }

    fn finish_bind(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<()> {
        let table = self.table_mut();
        let socket = table.get_resource_mut(&this)?;

        match socket.tcp_state {
            TcpState::BindStarted => {}
            _ => return Err(ErrorCode::NotInProgress.into()),
        }

        socket.tcp_state = TcpState::Bound;

        Ok(())
    }

    fn start_connect(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        network: Resource<Network>,
        remote_address: IpSocketAddress,
    ) -> SocketResult<()> {
        let table = self.table_mut();
        let r = {
            let socket = table.get_resource(&this)?;

            match socket.tcp_state {
                TcpState::Default => {}
                TcpState::Connected => return Err(ErrorCode::AlreadyConnected.into()),
                _ => return Err(ErrorCode::NotInProgress.into()),
            }

            let network = table.get_resource(&network)?;
            let connecter = network.pool.tcp_connecter(remote_address)?;

            // Do an OS `connect`. Our socket is non-blocking, so it'll either...
            {
                let view = &*socket.tcp_socket().as_socketlike_view::<TcpListener>();
                let r = connecter.connect_existing_tcp_listener(view);
                r
            }
        };

        match r {
            // succeed immediately,
            Ok(()) => {
                let socket = table.get_resource_mut(&this)?;
                socket.tcp_state = TcpState::ConnectReady;
                return Ok(());
            }
            // continue in progress,
            Err(err) if err.raw_os_error() == Some(INPROGRESS.raw_os_error()) => {}
            // or fail immediately.
            Err(err) => return Err(err.into()),
        }

        let socket = table.get_resource_mut(&this)?;
        socket.tcp_state = TcpState::Connecting;

        Ok(())
    }

    fn finish_connect(
        &mut self,
        this: Resource<tcp::TcpSocket>,
    ) -> SocketResult<(Resource<InputStream>, Resource<OutputStream>)> {
        let table = self.table_mut();
        let socket = table.get_resource_mut(&this)?;

        match socket.tcp_state {
            TcpState::ConnectReady => {}
            TcpState::Connecting => {
                // Do a `poll` to test for completion, using a timeout of zero
                // to avoid blocking.
                match rustix::event::poll(
                    &mut [rustix::event::PollFd::new(
                        socket.tcp_socket(),
                        rustix::event::PollFlags::OUT,
                    )],
                    0,
                ) {
                    Ok(0) => return Err(ErrorCode::WouldBlock.into()),
                    Ok(_) => (),
                    Err(err) => Err(err).unwrap(),
                }

                // Check whether the connect succeeded.
                match sockopt::get_socket_error(socket.tcp_socket()) {
                    Ok(Ok(())) => {}
                    Err(err) | Ok(Err(err)) => return Err(err.into()),
                }
            }
            _ => return Err(ErrorCode::NotInProgress.into()),
        };

        socket.tcp_state = TcpState::Connected;
        let (input, output) = socket.as_split();
        let input_stream = self.table_mut().push_child_resource(input, &this)?;
        let output_stream = self.table_mut().push_child_resource(output, &this)?;

        Ok((input_stream, output_stream))
    }

    fn start_listen(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<()> {
        let table = self.table_mut();
        let socket = table.get_resource_mut(&this)?;

        match socket.tcp_state {
            TcpState::Bound => {}
            TcpState::ListenStarted => return Err(ErrorCode::AlreadyListening.into()),
            TcpState::Connected => return Err(ErrorCode::AlreadyConnected.into()),
            _ => return Err(ErrorCode::NotInProgress.into()),
        }

        socket
            .tcp_socket()
            .as_socketlike_view::<TcpListener>()
            .listen(socket.listen_backlog_size)?;

        socket.tcp_state = TcpState::ListenStarted;

        Ok(())
    }

    fn finish_listen(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<()> {
        let table = self.table_mut();
        let socket = table.get_resource_mut(&this)?;

        match socket.tcp_state {
            TcpState::ListenStarted => {}
            _ => return Err(ErrorCode::NotInProgress.into()),
        }

        socket.tcp_state = TcpState::Listening;

        Ok(())
    }

    fn accept(
        &mut self,
        this: Resource<tcp::TcpSocket>,
    ) -> SocketResult<(
        Resource<tcp::TcpSocket>,
        Resource<InputStream>,
        Resource<OutputStream>,
    )> {
        let table = self.table();
        let socket = table.get_resource(&this)?;

        match socket.tcp_state {
            TcpState::Listening => {}
            TcpState::Connected => return Err(ErrorCode::AlreadyConnected.into()),
            _ => return Err(ErrorCode::NotInProgress.into()),
        }

        // Do the OS accept call.
        let tcp_socket = socket.tcp_socket();
        let (connection, _addr) = tcp_socket.try_io(Interest::READABLE, || {
            tcp_socket
                .as_socketlike_view::<TcpListener>()
                .accept_with(Blocking::No)
        })?;
        let mut tcp_socket = TcpSocket::from_tcp_stream(connection)?;

        // Mark the socket as connected so that we can exit early from methods like `start-bind`.
        tcp_socket.tcp_state = TcpState::Connected;

        let (input, output) = tcp_socket.as_split();
        let output: OutputStream = output;

        let tcp_socket = self.table_mut().push_resource(tcp_socket)?;
        let input_stream = self.table_mut().push_child_resource(input, &tcp_socket)?;
        let output_stream = self.table_mut().push_child_resource(output, &tcp_socket)?;

        Ok((tcp_socket, input_stream, output_stream))
    }

    fn local_address(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<IpSocketAddress> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        let addr = socket
            .tcp_socket()
            .as_socketlike_view::<std::net::TcpStream>()
            .local_addr()?;
        Ok(addr.into())
    }

    fn remote_address(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<IpSocketAddress> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        let addr = socket
            .tcp_socket()
            .as_socketlike_view::<std::net::TcpStream>()
            .peer_addr()?;
        Ok(addr.into())
    }

    fn address_family(
        &mut self,
        this: Resource<tcp::TcpSocket>,
    ) -> Result<IpAddressFamily, anyhow::Error> {
        let table = self.table();
        let socket = table.get_resource(&this)?;

        // If `SO_DOMAIN` is available, use it.
        //
        // TODO: OpenBSD also supports this; upstream PRs are posted.
        #[cfg(not(any(
            windows,
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        )))]
        {
            use rustix::net::AddressFamily;

            let family = sockopt::get_socket_domain(socket.tcp_socket())?;
            let family = match family {
                AddressFamily::INET => IpAddressFamily::Ipv4,
                AddressFamily::INET6 => IpAddressFamily::Ipv6,
                _ => return Err(ErrorCode::NotSupported.into()),
            };
            Ok(family)
        }

        // When `SO_DOMAIN` is not available, emulate it.
        #[cfg(any(
            windows,
            target_os = "ios",
            target_os = "macos",
            target_os = "netbsd",
            target_os = "openbsd"
        ))]
        {
            if let Ok(_) = sockopt::get_ipv6_unicast_hops(socket.tcp_socket()) {
                return Ok(IpAddressFamily::Ipv6);
            }
            if let Ok(_) = sockopt::get_ip_ttl(socket.tcp_socket()) {
                return Ok(IpAddressFamily::Ipv4);
            }
            Err(ErrorCode::NotSupported.into())
        }
    }

    fn ipv6_only(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<bool> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::get_ipv6_v6only(socket.tcp_socket())?)
    }

    fn set_ipv6_only(&mut self, this: Resource<tcp::TcpSocket>, value: bool) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::set_ipv6_v6only(socket.tcp_socket(), value)?)
    }

    fn set_listen_backlog_size(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        value: u64,
    ) -> SocketResult<()> {
        const MIN_BACKLOG: i32 = 1;
        const MAX_BACKLOG: i32 = i32::MAX; // OS'es will most likely limit it down even further.

        let table = self.table_mut();
        let socket = table.get_resource_mut(&this)?;

        // Silently clamp backlog size. This is OK for us to do, because operating systems do this too.
        let value = value
            .try_into()
            .unwrap_or(i32::MAX)
            .clamp(MIN_BACKLOG, MAX_BACKLOG);

        match socket.tcp_state {
            TcpState::Default | TcpState::BindStarted | TcpState::Bound => {
                // Socket not listening yet. Stash value for first invocation to `listen`.
                socket.listen_backlog_size = Some(value);

                Ok(())
            }
            TcpState::Listening => {
                // Try to update the backlog by calling `listen` again.
                // Not all platforms support this. We'll only update our own value if the OS supports changing the backlog size after the fact.

                rustix::net::listen(socket.tcp_socket(), value)
                    .map_err(|_| ErrorCode::AlreadyListening)?;

                socket.listen_backlog_size = Some(value);

                Ok(())
            }
            TcpState::Connected => Err(ErrorCode::AlreadyConnected.into()),
            TcpState::Connecting | TcpState::ConnectReady | TcpState::ListenStarted => {
                Err(ErrorCode::ConcurrencyConflict.into())
            }
        }
    }

    fn keep_alive(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<bool> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::get_socket_keepalive(socket.tcp_socket())?)
    }

    fn set_keep_alive(&mut self, this: Resource<tcp::TcpSocket>, value: bool) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::set_socket_keepalive(socket.tcp_socket(), value)?)
    }

    fn no_delay(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<bool> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::get_tcp_nodelay(socket.tcp_socket())?)
    }

    fn set_no_delay(&mut self, this: Resource<tcp::TcpSocket>, value: bool) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::set_tcp_nodelay(socket.tcp_socket(), value)?)
    }

    fn unicast_hop_limit(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<u8> {
        let table = self.table();
        let socket = table.get_resource(&this)?;

        // We don't track whether the socket is IPv4 or IPv6 so try one and
        // fall back to the other.
        match sockopt::get_ipv6_unicast_hops(socket.tcp_socket()) {
            Ok(value) => Ok(value),
            Err(Errno::NOPROTOOPT) => {
                let value = sockopt::get_ip_ttl(socket.tcp_socket())?;
                let value = value.try_into().unwrap();
                Ok(value)
            }
            Err(err) => Err(err.into()),
        }
    }

    fn set_unicast_hop_limit(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        value: u8,
    ) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;

        // We don't track whether the socket is IPv4 or IPv6 so try one and
        // fall back to the other.
        match sockopt::set_ipv6_unicast_hops(socket.tcp_socket(), Some(value)) {
            Ok(()) => Ok(()),
            Err(Errno::NOPROTOOPT) => Ok(sockopt::set_ip_ttl(socket.tcp_socket(), value.into())?),
            Err(err) => Err(err.into()),
        }
    }

    fn receive_buffer_size(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<u64> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::get_socket_recv_buffer_size(socket.tcp_socket())? as u64)
    }

    fn set_receive_buffer_size(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        value: u64,
    ) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        let value = value.try_into().map_err(|_| ErrorCode::OutOfMemory)?;
        Ok(sockopt::set_socket_recv_buffer_size(
            socket.tcp_socket(),
            value,
        )?)
    }

    fn send_buffer_size(&mut self, this: Resource<tcp::TcpSocket>) -> SocketResult<u64> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        Ok(sockopt::get_socket_send_buffer_size(socket.tcp_socket())? as u64)
    }

    fn set_send_buffer_size(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        value: u64,
    ) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;
        let value = value.try_into().map_err(|_| ErrorCode::OutOfMemory)?;
        Ok(sockopt::set_socket_send_buffer_size(
            socket.tcp_socket(),
            value,
        )?)
    }

    fn subscribe(&mut self, this: Resource<tcp::TcpSocket>) -> anyhow::Result<Resource<Pollable>> {
        crate::preview2::poll::subscribe(self.table_mut(), this)
    }

    fn shutdown(
        &mut self,
        this: Resource<tcp::TcpSocket>,
        shutdown_type: ShutdownType,
    ) -> SocketResult<()> {
        let table = self.table();
        let socket = table.get_resource(&this)?;

        let how = match shutdown_type {
            ShutdownType::Receive => std::net::Shutdown::Read,
            ShutdownType::Send => std::net::Shutdown::Write,
            ShutdownType::Both => std::net::Shutdown::Both,
        };

        socket
            .tcp_socket()
            .as_socketlike_view::<std::net::TcpStream>()
            .shutdown(how)?;
        Ok(())
    }

    fn drop(&mut self, this: Resource<tcp::TcpSocket>) -> Result<(), anyhow::Error> {
        let table = self.table_mut();

        // As in the filesystem implementation, we assume closing a socket
        // doesn't block.
        let dropped = table.delete_resource(this)?;

        // If we might have an `event::poll` waiting on the socket, wake it up.
        #[cfg(not(unix))]
        {
            match dropped.tcp_state {
                TcpState::Default
                | TcpState::BindStarted
                | TcpState::Bound
                | TcpState::ListenStarted
                | TcpState::ConnectReady => {}

                TcpState::Listening | TcpState::Connecting | TcpState::Connected => {
                    match rustix::net::shutdown(&*dropped.inner, rustix::net::Shutdown::ReadWrite) {
                        Ok(()) | Err(Errno::NOTCONN) => {}
                        Err(err) => Err(err).unwrap(),
                    }
                }
            }
        }

        drop(dropped);

        Ok(())
    }
}

// On POSIX, non-blocking TCP socket `connect` uses `EINPROGRESS`.
// <https://pubs.opengroup.org/onlinepubs/9699919799/functions/connect.html>
#[cfg(not(windows))]
const INPROGRESS: Errno = Errno::INPROGRESS;

// On Windows, non-blocking TCP socket `connect` uses `WSAEWOULDBLOCK`.
// <https://learn.microsoft.com/en-us/windows/win32/api/winsock2/nf-winsock2-connect>
#[cfg(windows)]
const INPROGRESS: Errno = Errno::WOULDBLOCK;
