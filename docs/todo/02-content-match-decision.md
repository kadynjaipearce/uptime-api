# 2. Decide what a content mismatch means, and carry it through

Depends on #1 being done (need `content_matched` to actually exist first).

- [ ] Decide: does a content mismatch just get reported (temporary signal,
      no schema change), or does it make the check a failure
      (`success = false`) and/or get persisted per-check (needs a migration)?
- [ ] Add a `content_matched` field to `CheckResult`
      (`core/handler/check.rs:16-28`) — currently has no slot for it.
- [ ] Set it in `run_check` after the HTTP stage runs.
- [ ] If a mismatch should count as a failed check: add a fifth
      `ErrorStage` variant (e.g. `ContentMismatch`) alongside the existing
      `Dns/Connect/Tls/Http` (`check.rs:6-12`), so `error_stage` in the DB
      can distinguish "never got a response" from "responded, wrong content."
- [ ] If it's only meant to feed the snapshot/alert logic (see #3) and
      shouldn't affect `success`: skip touching `ErrorStage` entirely, just
      thread `content_matched` through as its own field.
- [ ] If persisting per-check: add the column via migration, update
      `RecordCheck` / `CheckRow`.
