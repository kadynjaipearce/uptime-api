# 3. Wire snapshot compare/save into the worker — DONE

`Database::record_content_snapshot` and `Database::latest_content_snapshot`
(`src/database/models/content_snapshot.rs`) are now called from
`Worker::process_check` (`src/core/worker.rs`):

- [x] Fetches `latest_content_snapshot(url_id)`.
- [x] Compares it against this check's `content_hash` (a `match` producing
      the hash to save, `None` when unchanged — avoids `unwrap()`).
- [x] Calls `record_content_snapshot` when the hash differs or there was no
      previous snapshot.

**Scope question resolved:** content drift alone does *not* raise an
incident. Incidents are keyed off `content_matched` (from #2) instead —
`migrations/20260720111700_create_incidents.sql` is now in use, with a
follow-up migration (`20260913132408_fix_incidents_resolved_at_nullable.sql`)
making `resolved_at` nullable so an open incident can be inserted before
it's resolved. `src/database/models/incident.rs` /
`src/http/incident.rs` hold the row type and `open_incident` /
`resolve_incident` / `latest_open_incident` / `list_open_incidents_for_url`
queries.

## Still open

- [ ] No tests yet for the incident state-transition `match` or the
      snapshot-diff `match` in `worker.rs` (the pure logic in
      `core/handler/http.rs::parse_response` is covered, but the worker-level
      wiring isn't).
- [ ] Haven't run the full pipeline against a real target end-to-end yet to
      confirm probe -> snapshot -> incident behaves as expected.
- [ ] `Content-Length` / chunked body handling still not implemented (see
      #1's known limitation).
