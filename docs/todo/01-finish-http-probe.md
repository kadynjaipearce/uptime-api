# 1. Finish `probe()` in src/core/handler/http.rs

Blocks compilation right now (`unimplemented!()`). Do this one first.

- [ ] Fix the `\r\n\r\n` split: get the status-line bytes and a clean body as
      two separate slices. `drain(0..pos)` currently leaves the `\r\n\r\n`
      stuck on the front of the body — drain `0.. pos + 4` instead, and keep
      the header slice around instead of throwing it away.
- [ ] Parse `status_code: u16` out of the status line (`HTTP/1.1 200 OK` →
      `200`).
- [ ] Add a hashing crate to `Cargo.toml` (nothing like `sha2`/`blake3` is
      there yet). `blake3` is a good default — fast, and gives you a hex
      string in one call.
- [ ] Hash the body bytes -> `content_hash`.
- [ ] Un-underscore `expected_content` and compare it against the body (as
      lossy UTF-8) -> `content_matched: Option<bool>` (`None` if no expected
      content was given, `Some(bool)` otherwise).
- [ ] Return `Ok(ProbeResponse { status_code, content_hash, content_matched })`
      instead of `unimplemented!()`.

**Known limitation, ok to leave for now:** body is read as "whatever's left
after headers, until EOF" — relies on `Connection: close`. Doesn't handle
`Content-Length` or `Transfer-Encoding: chunked`. Fine for a first working
version; revisit if a real target server needs it.
