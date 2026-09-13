# 2. Decide what a content mismatch means, and carry it through — DONE

**Decision:** a mismatch does not fail the check (`success` is unaffected,
no new `ErrorStage`, no migration on `checks`/`CheckRow`). Instead it drives
an incident lifecycle (see #3).

- [x] `content_matched: Option<bool>` added to `CheckResult`
      (`core/handler/check.rs`).
- [x] Set in `run_check` from `response.content_matched` after the HTTP
      stage runs.
- [x] No `ErrorStage::ContentMismatch` — mismatch is tracked separately via
      the `incidents` table instead of the `checks` row.
- [x] `Worker::process_check` (`core/worker.rs`) opens an incident when
      `content_matched` goes from matching/unknown to `Some(false)` with no
      open incident yet, and resolves it when `content_matched` becomes
      `Some(true)` while one is open. See #3 for the incidents-table details.
