use std::io::Result;
use std::net;
use std::net::SocketAddr;

use ntex_bytes::PoolRef;
use ntex_io::Io;

mod io;

//pub use self::io::{SocketOptions, TokioIoBoxed};

struct TcpStream(folo::net::TcpConnection);

/// Opens a TCP connection to a remote host.
pub async fn tcp_connect(addr: SocketAddr) -> Result<Io> {
    let sock = std::net::TcpStream::connect(addr)?;
    sock.set_nodelay(true)?;
    let folo_socket: folo::net::TcpConnection = sock.into();
    Ok(Io::new(TcpStream(folo_socket)))
}

/// Opens a TCP connection to a remote host and use specified memory pool.
pub async fn tcp_connect_in(addr: SocketAddr, pool: PoolRef) -> Result<Io> {
    let sock = std::net::TcpStream::connect(addr)?;
    sock.set_nodelay(true)?;
    let folo_socket: folo::net::TcpConnection = sock.into();
    Ok(Io::with_memory_pool(TcpStream(folo_socket), pool))
}

/// Convert std TcpStream to tokio's TcpStream
pub fn from_tcp_stream(stream: net::TcpStream) -> Result<Io> {
    stream.set_nonblocking(true)?;
    stream.set_nodelay(true)?;
    let folo_socket: folo::net::TcpConnection = stream.into();
    Ok(Io::new(TcpStream(folo_socket)))
}
