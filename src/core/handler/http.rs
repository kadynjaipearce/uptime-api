use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::core::handler::tls::TlsHandshakeOutcome;

/// Result of the HTTP stage: enough to fill in `checks.status_code` and
/// `checks.content_hash`, and to compare against a URL's expected content.
///
/// Design decisions (see check.rs::RecordCheck, which this feeds):
/// - `content_hash` is a hash of the response BODY ONLY (not headers),
///   computed unconditionally on every successful probe — independent of
///   whether `expected_content` was set. It's a snapshot fingerprint: the
///   caller can diff it against the previous check's `content_hash` for the
///   same url_id to detect silent content drift over time, separate from
///   the expected_content substring check below.
/// - `content_matched` (to be added) carries the result of the
///   expected_content check: `None` when no expected_content was supplied,
///   `Some(true/false)` when it was. probe() itself does NOT decide
///   success/failure — it only reports facts. The caller in check.rs is
///   responsible for combining status_code + content_matched into
///   `RecordCheck.success` / `error_stage` / `error_message`.
pub struct ProbeResponse {
    pub status_code: u16,
    pub content_hash: String,
    pub content_matched: Option<bool>,
}

/// Sends the HTTP request over `stream` and reads the response, timed from
/// first byte sent to first byte received so its duration lands in
/// `checks.ttfb_ms`. Compares the body against `expected_content` if set.
///
/// Planned flow:
/// 1. Write the request, flush, read the raw response into a buffer.
/// 2. Parse the raw response into status line / headers / body instead of
///    treating it as one opaque blob — status_code comes from the status
///    line, and both hashing and expected_content matching should run
///    against the body only (headers can legitimately contain arbitrary
///    text that would produce false matches/mismatches otherwise).
/// 3. Hash the body unconditionally -> ProbeResponse.content_hash.
/// 4. If expected_content is Some, check body.contains(content) ->
///    ProbeResponse.content_matched; if None, content_matched stays None.
/// 5. Return status_code + content_hash + content_matched; leave the
///    success/error_stage decision to the caller (check.rs), which has
///    the full picture (this stage's result plus dns/connect/tls stages).
pub async fn probe(
    mut stream: TlsHandshakeOutcome,
    domain: &str,
    expected_content: Option<&str>,
) -> Result<ProbeResponse, anyhow::Error> {
    //
    let request = format!(
        "GET / HTTP/1.1\r\n
         Host: {}\r\n
         User-Agent: rust-tcpstream\r\n
         Connection: close\r\n
         \r\n",
        domain
    );

    let _ = &stream.stream.write_all(request.as_bytes());
    let _ = &stream.stream.flush();

    let mut response = String::new();
    #[allow(clippy::let_underscore_future)]
    let _ = stream.stream.read_to_string(&mut response);

    let _found = match expected_content {
        Some(content) => response.contains(content),
        None => false,
    };

    let _ = (stream, domain, expected_content);
    unimplemented!()
}
