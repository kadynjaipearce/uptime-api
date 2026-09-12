# 3. Wire snapshot compare/save into the worker

Independent of #2 — only needs a `content_hash` from #1, doesn't need #2
resolved in any particular shape.

`Database::record_content_snapshot` and `Database::latest_content_snapshot`
(`src/database/models/content_snapshot.rs`) already exist and work. Nothing
calls them yet.

- [ ] In `Worker::process_check` (`src/core/worker.rs:99-134`), after
      `check::run_check` returns:
  - [ ] Call `latest_content_snapshot(url_id)` to get the previous hash (if
        any).
  - [ ] Compare it to this check's `content_hash`.
  - [ ] If different (or no previous snapshot exists), call
        `record_content_snapshot` to save the new one.
- [ ] Open scope question: should content drift alone raise an incident?
      There's an unused migration (`migrations/20260720111700_create_incidents.sql`)
      but nothing in `worker.rs` writes to it yet. Decide whether drift alone
      creates an incident, or only when combined with `success`/
      `content_matched` from #2.

**Suggested order overall:** #1 → #2 → #3. #1 unblocks compilation, #2
decides whether a migration is needed at all, #3 is worker-loop wiring that
can happen independently once #1 produces a real `content_hash`.
