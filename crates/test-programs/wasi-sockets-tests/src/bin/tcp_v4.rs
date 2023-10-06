//! A simple TCP testcase, using IPv4.

use wasi::io::poll;
use wasi::sockets::network::{IpAddressFamily, IpSocketAddress, Ipv4SocketAddress};
use wasi::sockets::{instance_network, tcp_create_socket};
use wasi_sockets_tests::*;

fn main() {
    let net = instance_network::instance_network();

    let sock = tcp_create_socket::create_tcp_socket(IpAddressFamily::Ipv4).unwrap();

    let addr = IpSocketAddress::Ipv4(Ipv4SocketAddress {
        port: 0,                 // use any free port
        address: (127, 0, 0, 1), // localhost
    });

    let sub = sock.subscribe();

    sock.start_bind(&net, addr).unwrap();

    poll::poll_one(&sub);
    drop(sub);

    sock.finish_bind().unwrap();

    example_body(net, sock, IpAddressFamily::Ipv4)
}
