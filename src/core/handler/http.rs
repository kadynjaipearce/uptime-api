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
/// unit tested without a real socket.
fn parse_response(
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn response(status_line: &str, body: &str) -> Vec<u8> {
        format!("{status_line}\r\nContent-Type: text/plain\r\n\r\n{body}").into_bytes()
    }

    #[test]
    fn splits_header_and_body_without_leftover_separator() {
        let buf = response("HTTP/1.1 200 OK", "hello world");
        let result = parse_response(buf, None).unwrap();

        assert_eq!(
            result.content_hash,
            blake3::hash(b"hello world").to_string()
        );
    }

    #[test]
    fn parses_status_code_from_status_line() {
        let buf = response("HTTP/1.1 404 Not Found", "missing");
        let result = parse_response(buf, None).unwrap();

        assert_eq!(result.status_code, 404);
    }

    #[test]
    fn no_expected_content_means_no_match_result() {
        let buf = response("HTTP/1.1 200 OK", "hello world");
        let result = parse_response(buf, None).unwrap();

        assert_eq!(result.content_matched, None);
    }

    #[test]
    fn expected_content_present_matches() {
        let buf = response("HTTP/1.1 200 OK", "hello world");
        let result = parse_response(buf, Some("world")).unwrap();

        assert_eq!(result.content_matched, Some(true));
    }

    #[test]
    fn expected_content_missing_does_not_match() {
        let buf = response("HTTP/1.1 200 OK", "hello world");
        let result = parse_response(buf, Some("goodbye")).unwrap();

        assert_eq!(result.content_matched, Some(false));
    }

    #[test]
    fn missing_header_body_separator_is_an_error() {
        let buf = b"HTTP/1.1 200 OK\r\nContent-Type: text/plain".to_vec();

        assert!(parse_response(buf, None).is_err());
    }

    #[test]
    fn malformed_status_line_is_an_error() {
        let buf = response("garbage", "body");

        assert!(parse_response(buf, None).is_err());
    }

    #[test]
    fn same_body_hashes_the_same_regardless_of_status() {
        let ok = parse_response(response("HTTP/1.1 200 OK", "hello world"), None).unwrap();
        let not_found =
            parse_response(response("HTTP/1.1 404 Not Found", "hello world"), None).unwrap();

        assert_eq!(ok.content_hash, not_found.content_hash);
    }
}
