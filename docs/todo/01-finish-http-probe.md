# 1. Finish `probe()` in src/core/handler/http.rs — DONE

- [x] Fixed the `\r\n\r\n` split — header and body are now clean, separate
      slices.
- [x] Parses `status_code: u16` out of the status line.
- [x] Added `blake3` to `Cargo.toml` and hashes the body -> `content_hash`.
- [x] Compares `expected_content` against the body (lossy UTF-8) ->
      `content_matched: Option<bool>`.
- [x] Returns `Ok(ProbeResponse { status_code, content_hash, content_matched })`.
- [x] Unit tests added in `http.rs` (`parse_response` pulled out as a pure
      function so it can be tested without a real socket): header/body
      split, status parsing, match/no-match, malformed input errors, hash
      stability.

**Known limitation, still true:** body is read as "whatever's left after
headers, until EOF" — relies on `Connection: close`. Doesn't handle
`Content-Length` or `Transfer-Encoding: chunked`. Fine for now; revisit if a
real target server needs it.
