# Where I left off — HTTP probe + content hashing

Picking this back up after a break? Start here. Branch: `feature/http/probe`.

## The big picture

The check pipeline goes DNS → TCP → TLS → HTTP. The last step — actually
reading the HTTP response and doing something with it — is the part that's
half-built. Everything before it (DNS, TCP, TLS) works.

## Status: #1, #2, #3 all done

- **#1 — `src/core/handler/http.rs`:** `probe()` sends the request, reads
  the response, cleanly splits headers/body, parses the status code, hashes
  the body with `blake3`, and compares it against `expected_content`. Pure
  parsing logic lives in `parse_response` (split out so it's unit-testable
  without a real socket) and has test coverage. Known limitation still
  standing: body reading relies on `Connection: close` / read-to-EOF, no
  `Content-Length` or chunked-encoding support — fine for now.
- **#2 — content match decision:** resolved as "informational, doesn't fail
  the check." `content_matched` flows through `CheckResult` without
  touching `success`/`ErrorStage`. It instead drives an incident lifecycle
  (see #3).
- **#3 — snapshot + incidents wiring:** `Worker::process_check` now calls
  `latest_content_snapshot`/`record_content_snapshot` to track hash changes,
  and separately opens/resolves rows in the `incidents` table based on
  `content_matched` transitions (mismatch with none open -> `open_incident`;
  match again with one open -> `resolve_incident`). See
  `src/database/models/incident.rs` and `src/http/incident.rs`.

## What's left

- [ ] Tests for the worker-level `match` logic (snapshot diff, incident
      open/resolve transitions) — only the `http.rs` parsing logic has tests
      so far.
- [ ] Run the full pipeline against a real target end-to-end to confirm
      probe -> snapshot -> incident behaves as expected in practice.
- [ ] `Content-Length` / chunked body handling, if a real target needs it.

## Side note: deployment

Cloudflare Workers won't work for this project — it opens raw TCP/UDP
connections (for the TLS handshake and custom DNS lookups) and runs a
background loop that polls forever, and Workers' sandbox doesn't allow
either of those things. Something like Fly.io, Railway, Shuttle.rs, or a
plain VPS is the right fit instead, since they let you run a normal
long-lived Rust process with real network sockets.
