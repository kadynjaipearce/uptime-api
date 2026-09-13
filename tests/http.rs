#![allow(clippy::unwrap_used, clippy::expect_used)]

use uptime_api::core::handler::http::parse_response;

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
    let not_found = parse_response(response("HTTP/1.1 404 Not Found", "hello world"), None)
        .unwrap();

    assert_eq!(ok.content_hash, not_found.content_hash);
}
