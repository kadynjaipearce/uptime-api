# Where I left off — HTTP probe + content hashing

Picking this back up after a break? Start here. Branch: `feature/http/probe`.

## The big picture

The check pipeline goes DNS → TCP → TLS → HTTP. The last step — actually
reading the HTTP response and doing something with it — is the part that's
half-built. Everything before it (DNS, TCP, TLS) works.

## 1. `src/core/handler/http.rs` — this is the main gap

This function is supposed to: send the request, read the response, pull out
the status code, hash the page content, and check it against what the user
expects to see. Right now it stops partway through and just crashes
(`unimplemented!()`) instead of returning a result.

Here's what's done vs. not, step by step:

- ✅ Sends the GET request over the TLS connection.
- ✅ Reads the raw response bytes back.
- ⚠️ **Splits headers from body — but it's buggy.** HTTP responses look like
  `HEADERS\r\n\r\nBODY`. The code finds where `\r\n\r\n` sits, but then cuts
  the buffer at the wrong spot — it leaves the `\r\n\r\n` stuck onto the
  front of the body instead of removing it. Small fix, but it means the body
  isn't clean yet.
- ❌ **Never reads the status code.** The headers (which contain the
  `200 OK` / `404 Not Found` line) get read but nothing pulls the status
  code out of them before they're thrown away.
- ❌ **Doesn't handle real-world response shapes.** Some servers send the
  body in "chunks" (`Transfer-Encoding: chunked`) instead of one block, and
  some tell you exactly how many bytes to expect (`Content-Length`). Right
  now the code just treats "whatever bytes are left" as the body, which
  will work for simple sites but break or hash garbage on others.
- ❌ **No hashing at all yet.** There's no hashing library installed in the
  project (checked `Cargo.toml` — nothing like `sha2` is there). So even
  though the code has a `content_hash` field ready to fill in, nothing
  actually computes a hash of the page content.
- ❌ **Never compares against the expected content.** The function takes in
  what the user expects the page to contain, but it's currently named with
  an underscore (`_expected_content`), which in Rust is a way of saying
  "I know I'm not using this yet." Nothing compares it to the real page.
- ❌ **Doesn't return anything** — it just stops and crashes instead of
  handing back the status code, hash, and match result.

**Suggested order to tackle this:**
1. Fix the header/body split bug.
2. Pull the status code out of the header text before discarding it.
3. Handle `Content-Length` (and ideally chunked responses) so the body is
   accurate.
4. Add a hashing library to the project and hash the cleaned-up body.
5. Compare the result against the expected content.
6. Return the finished result instead of crashing.

## 2. The "did the content match?" result gets thrown away

Even once step 1 is fixed, there's a second gap: the result of "did the page
match what we expected" doesn't actually go anywhere yet.

- The place that stores the overall check result (`CheckResult`, in
  `core/handler/check.rs`) doesn't have a slot for "matched or not" — only
  for the hash itself. So that piece of information would be calculated and
  then immediately dropped.
- The database side (`CheckRow` / `RecordCheck`) also has no column for it.
  So there's currently nowhere to save "yes this matched" or "no it didn't"
  even if you wanted to.

**Decision needed:** do you want to permanently store "matched: yes/no" for
every single check in the database (needs a database migration to add a
column), or is it only meant to be a temporary flag used in the moment to
decide "should I raise an alert" (no migration needed, just wiring)?

## 3. Nothing compares snapshots over time yet

There's a separate table/feature for "content snapshots" — basically a
history of hashes over time so you can tell when a page's content changed.
The database functions to save a snapshot and fetch the most recent one
already exist and work fine on their own, but **nothing in the app calls
them**. They're just sitting there unused.

The missing piece is the logic that would live in the background worker
(`core/worker.rs`): after each check runs, look up the last saved hash for
that URL, compare it to the new one, and if it's different, save a new
snapshot (and probably raise an alert). None of that "look up → compare →
save" sequence has been written yet — this is separate work from fixing
`http.rs`, not something that comes for free once that's done.

## Quick checklist for next time

- [ ] Fix the drain bug so the body doesn't include stray `\r\n\r\n`
- [ ] Parse the status line for `status_code`
- [ ] Handle `Content-Length` / chunked bodies properly
- [ ] Add a hashing crate and compute `content_hash`
- [ ] Compare against `expected_content` to compute a match result
- [ ] Return `ProbeResponse` (delete the `unimplemented!()`)
- [ ] Decide if "matched or not" gets saved to the database or stays temporary
- [ ] Wire up the snapshot save/compare logic in the worker's check loop

## Side note: deployment

Cloudflare Workers won't work for this project — it opens raw TCP/UDP
connections (for the TLS handshake and custom DNS lookups) and runs a
background loop that polls forever, and Workers' sandbox doesn't allow
either of those things. Something like Fly.io, Railway, Shuttle.rs, or a
plain VPS is the right fit instead, since they let you run a normal
long-lived Rust process with real network sockets.
