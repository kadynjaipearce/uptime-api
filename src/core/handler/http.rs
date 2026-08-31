use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::handler::tls::TlsHandshakeOutcome;

const MAX_RESPONSE_SIZE: usize = 10 * 1024 * 1024; // max 10mb

pub struct ProbeResponse {
    pub status_code: u16,
    pub content_hash: String,
    pub content_matched: Option<bool>,
}

pub async fn probe(
    mut stream: TlsHandshakeOutcome,
    domain: &str,
    _expected_content: Option<&str>,
) -> Result<ProbeResponse, anyhow::Error> {
    let request = format!(
        "GET / HTTP/1.1\r\nHost: {}\r\nUser-Agent: rust-uptimeapi\r\nConnection: close\r\n\r\n",
        domain
    );

    stream.tcp.write_all(request.as_bytes()).await?;
    stream.tcp.flush().await?;

    let mut response_buf: Vec<u8> = Vec::new();

    let bytes_read = stream
        .tcp
        .take(MAX_RESPONSE_SIZE as u64)
        .read_to_end(&mut response_buf)
        .await?;

    if bytes_read == MAX_RESPONSE_SIZE {
        anyhow::bail!("Response exceeds maximum allowed size of 10mb")
    }

    if let Some(pos) = response_buf.windows(4).position(|w| w == b"\r\n\r\n") {
        response_buf.drain(0..pos);
    }

    unimplemented!()
}
