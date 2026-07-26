use tokio::net::TcpStream;

/// Performs the TLS handshake over an already-connected `stream`, timed
/// separately so its duration lands in `checks.tls_ms`. Returns the
/// negotiated stream ready for the HTTP stage to write/read over.
pub async fn handshake(stream: TcpStream, domain: &str) -> Result<TcpStream, anyhow::Error> {
    let _ = (stream, domain);
    unimplemented!()
}
