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
    expected_content: Option<&str>,
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

    parse_response(response_buf, expected_content)
}

/// Splits headers from body, parses the status line, hashes the body, and
/// checks it against `expected_content`. Pulled out of `probe` so it can be
/// tested without a real socket.
pub fn parse_response(
    mut response_buf: Vec<u8>,
    expected_content: Option<&str>,
) -> Result<ProbeResponse, anyhow::Error> {
    let pos = response_buf
        .windows(4)
        .position(|w| w == b"\r\n\r\n")
        .ok_or_else(|| anyhow::anyhow!("response missing header/body separator"))?;

    let header_bytes: Vec<u8> = response_buf.drain(0..pos).collect();
    response_buf.drain(0..4);

    let response_header = String::from_utf8_lossy(&header_bytes);
    let response_body = String::from_utf8_lossy(&response_buf);

    let status_code: u16 = response_header
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .ok_or_else(|| anyhow::anyhow!("malformed status line"))?
        .parse()?;

    let content_matched: Option<bool> =
        expected_content.map(|content| response_body.contains(content));

    Ok(ProbeResponse {
        status_code,
        content_matched,
        content_hash: blake3::hash(&response_buf).to_string(), // hash the body of the response for comparison
    })
}
