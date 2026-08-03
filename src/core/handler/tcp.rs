use std::net::{IpAddr, SocketAddr};

use chrono::Duration;
use tokio::net::TcpStream;

const CONNECT_TIMEOUT: Duration = Duration::seconds(2);

const CONNECTION_PORT: u16 = 443;

async fn connect_tcp(ip: IpAddr, port: u16) -> Result<TcpStream, anyhow::Error> {
    let addr = SocketAddr::new(ip, port);
    let stream = tokio::time::timeout(CONNECT_TIMEOUT.to_std()?, TcpStream::connect(addr))
        .await
        .map_err(|_| anyhow::anyhow!("TCP connect to {addr} timed out"))??;

    Ok(stream)
}

/// Opens a TCP connection to `ip`, timed separately from DNS so its
/// duration lands in `checks.connect_ms`.
pub async fn connect(ip: IpAddr) -> Result<TcpStream, anyhow::Error> {
    connect_tcp(ip, CONNECTION_PORT).await
}
