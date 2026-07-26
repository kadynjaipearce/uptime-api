use std::net::IpAddr;

use tokio::net::TcpStream;

/// Opens a TCP connection to `ip`, timed separately from DNS so its
/// duration lands in `checks.connect_ms`.
pub async fn connect(ip: IpAddr) -> Result<TcpStream, anyhow::Error> {
    let _ = ip;
    unimplemented!()
}
