use std::{net::Ipv4Addr, time::Instant};

use super::{dns, http, tcp, tls};

/// Which stage a check failed at, if any.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStage {
    Dns,
    Connect,
    Tls,
    Http,
}

/// Outcome of running all stages for one region against one URL.
#[derive(Debug, Default)]
pub struct CheckResult {
    pub dns_ms: Option<u32>,
    pub connect_ms: Option<u32>,
    pub tls_ms: Option<u32>,
    pub ttfb_ms: Option<u32>,
    pub total_ms: u32,

    pub status_code: Option<u16>,
    pub success: bool,
    pub error_stage: Option<ErrorStage>,
    pub error_message: Option<String>,
    pub content_hash: Option<String>,
}

/// Runs the DNS -> TCP -> TLS -> HTTP pipeline for a single check job.
/// Stops at the first stage that fails and records where/why
pub async fn run_check(
    domain: &str,
    expected_content: Option<&str>,
    dns_servers: &[Ipv4Addr],
) -> CheckResult {
    let started = Instant::now();
    let mut result = CheckResult::default();

    let ip = match time_stage(&mut result.dns_ms, || dns::resolve(domain, dns_servers)).await {
        Ok(ip) => ip,
        Err(err) => return fail(result, started, ErrorStage::Dns, err),
    };

    let stream = match time_stage(&mut result.connect_ms, || tcp::connect(ip)).await {
        Ok(stream) => stream,
        Err(err) => return fail(result, started, ErrorStage::Connect, err),
    };

    let tls_stream = match time_stage(&mut result.tls_ms, || tls::handshake(stream, domain)).await {
        Ok(tls_stream) => tls_stream,
        Err(err) => return fail(result, started, ErrorStage::Tls, err),
    };

    let response = match time_stage(&mut result.ttfb_ms, || {
        http::probe(tls_stream, domain, expected_content)
    })
    .await
    {
        Ok(response) => response,
        Err(err) => return fail(result, started, ErrorStage::Http, err),
    };

    result.status_code = Some(response.status_code);
    result.content_hash = Some(response.content_hash);
    result.success = true;
    result.total_ms = elapsed_ms(started);
    result
}

/// Runs one stage's future, recording its wall-clock time into `slot`
/// regardless of whether the stage succeeds or fails.
async fn time_stage<T, E, F, Fut>(slot: &mut Option<u32>, stage: F) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
{
    let started = Instant::now();
    let outcome = stage().await;
    *slot = Some(elapsed_ms(started));
    outcome
}

fn fail(
    mut result: CheckResult,
    started: Instant,
    stage: ErrorStage,
    err: impl ToString,
) -> CheckResult {
    result.success = false;
    result.error_stage = Some(stage);
    result.error_message = Some(err.to_string());
    result.total_ms = elapsed_ms(started);
    result
}

fn elapsed_ms(started: Instant) -> u32 {
    started.elapsed().as_millis().min(u32::MAX as u128) as u32
}
