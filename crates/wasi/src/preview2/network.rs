use crate::preview2::bindings::sockets::network::{Ipv4Address, Ipv6Address};
use crate::preview2::bindings::wasi::sockets::network::ErrorCode;
use crate::preview2::TrappableError;
use cap_std::net::Pool;
use std::sync::Arc;

pub struct Network {
    pub pool: Arc<Pool>,
    pub allow_ip_name_lookup: bool,
}

pub type SocketResult<T> = Result<T, SocketError>;

pub type SocketError = TrappableError<ErrorCode>;

impl From<wasmtime::component::ResourceTableError> for SocketError {
    fn from(error: wasmtime::component::ResourceTableError) -> Self {
        Self::trap(error)
    }
}

impl From<std::io::Error> for SocketError {
    fn from(error: std::io::Error) -> Self {
        ErrorCode::from(error).into()
    }
}

impl From<rustix::io::Errno> for SocketError {
    fn from(error: rustix::io::Errno) -> Self {
        ErrorCode::from(error).into()
    }
}

#[derive(Copy, Clone)]
pub enum SocketAddressFamily {
    Ipv4,
    Ipv6 { v6only: bool },
}

pub(crate) fn to_ipv4_addr(addr: Ipv4Address) -> std::net::Ipv4Addr {
    let (x0, x1, x2, x3) = addr;
    std::net::Ipv4Addr::new(x0, x1, x2, x3)
}

pub(crate) fn from_ipv4_addr(addr: std::net::Ipv4Addr) -> Ipv4Address {
    let [x0, x1, x2, x3] = addr.octets();
    (x0, x1, x2, x3)
}

pub(crate) fn to_ipv6_addr(addr: Ipv6Address) -> std::net::Ipv6Addr {
    let (x0, x1, x2, x3, x4, x5, x6, x7) = addr;
    std::net::Ipv6Addr::new(x0, x1, x2, x3, x4, x5, x6, x7)
}

pub(crate) fn from_ipv6_addr(addr: std::net::Ipv6Addr) -> Ipv6Address {
    let [x0, x1, x2, x3, x4, x5, x6, x7] = addr.segments();
    (x0, x1, x2, x3, x4, x5, x6, x7)
}
