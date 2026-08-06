use crate::core::handler::tls::TlsHandshakeOutcome;

/// Result of the HTTP stage: enough to fill in `checks.status_code` and
/// `checks.content_hash`, and to compare against a URL's expected content.
pub struct ProbeResponse {
    pub status_code: u16,
    pub content_hash: String,
}

/// Sends the HTTP request over `stream` and reads the response, timed from
/// first byte sent to first byte received so its duration lands in
/// `checks.ttfb_ms`. Compares the body against `expected_content` if set.
pub async fn probe(
    stream: TlsHandshakeOutcome,
    domain: &str,
    expected_content: Option<&str>,
) -> Result<ProbeResponse, anyhow::Error> {
    let _ = (stream, domain, expected_content);
    unimplemented!()
}
