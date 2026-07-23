# Uptime API

Uptime and content-integrity monitoring for the sites you host for clients — built to catch not just "is it down," but "is it actually right."

## Why

Most uptime checkers fire a GET request and call `200 OK` healthy. That's not always true. DNS can resolve to the wrong host, TLS can terminate early, a botched deploy or a compromised site can serve the same status code with completely different content, and none of that shows up in a simple status check. If you're responsible for keeping client sites reachable *and correct*, a green checkmark on a raw HTTP status isn't enough.

This project traces each check manually through the full path — DNS resolution, TCP connect, TLS handshake, TTFB, full response — instead of treating the request as a black box, then verifies a unique, expected string is present in the response body. That's what catches "resolves and returns 200, but it's the wrong site" or "still 200, but the content's been replaced," not just "the server didn't respond."

Checks are run from multiple AWS regions, so an alert tells you whether an outage is global, regional, or a DNS propagation issue — not just "something, somewhere, is wrong."

## How it works

- A Postgres table of monitored URLs, one row per site, each with its own check interval and expected content string
- A scheduler ticks on an interval, claims due URLs, and fans each one out into one job per configured region
- Region workers *(in progress)* run the actual probe — DNS → connect → TLS → TTFB → full body — and report per-stage timings back
- The response body is checked against the configured expected content string, so "200 OK, wrong content" counts as a failure, not a pass
- Incidents open automatically when checks start failing and resolve when they recover

## Status

Early and actively in progress. Right now this is **API-only** — no frontend yet.

**Roadmap:** self-hosted API → hosted micro-SaaS with an open-source core API.

### Working

- [x] Axum + Postgres (`sqlx`) project scaffold, migrations for `url`, `jobs`, `checks`, `incidents`, `content_snapshots`
- [x] Scheduler skeleton — polls due URLs on a tick, enqueues one job per configured region
- [x] `POST /url` — register a URL to monitor, with domain/name/interval validation
- [x] `GET /url/:id` — fetch a monitored URL
- [x] `PATCH /url/:id` — partial update
- [x] `DELETE /url/:id`
- [x] Uniform JSON envelope for success and error responses
- [x] Validation test coverage for URL payloads

### Not built yet

- [ ] The actual DNS → TCP → TLS → HTTP trace / probe logic (region workers)
- [ ] `GET /url` — list all monitored URLs
- [ ] Recording check results (`POST /checks`) — jobs are enqueued but nothing consumes them yet
- [ ] Incident open/resolve endpoints (currently a read-only stub)
- [ ] Content snapshot recording endpoint
- [ ] Multi-region AWS deployment
- [ ] Auth / API keys
- [ ] Frontend
- [ ] Hosted micro-SaaS offering

## Stack

| Layer | Choice |
|---|---|
| Language | Rust |
| HTTP | Axum |
| Database | Postgres via `sqlx` |
| Runtime | Tokio |
| Deploy target | AWS, multi-region |

## Getting started

```bash
cp .env.example .env
# fill in DATABASE_URL, VERSION_SLUG, PORT, REGIONS, FRONTEND_URL

cargo run
```

Migrations run automatically on startup against `DATABASE_URL`.

### Environment variables

| Variable | Required | Default | Notes |
|---|---|---|---|
| `DATABASE_URL` | yes | — | Postgres connection string |
| `VERSION_SLUG` | yes | — | API path prefix, e.g. `v1` → `/api/v1/...` |
| `FRONTEND_URL` | yes | — | Allowed CORS origin |
| `PORT` | no | `8080` | |
| `MAX_CONNECTIONS` | no | `1` | Postgres pool size |
| `REGIONS` | yes | — | Comma-separated, e.g. `us-east,eu-west` |

## API (current)

Base path: `/api/{VERSION_SLUG}/`

| Method | Path | Status |
|---|---|---|
| GET | `/health` | working |
| POST | `/url` | working |
| GET | `/url/:id` | working |
| PATCH | `/url/:id` | working |
| DELETE | `/url/:id` | working |
| GET | `/url/:id/incidents` | stub |
| GET | `/url/:id/checks` | stub |

## Testing

```bash
cargo test
```

## License

TBD — the core API is planned to be open source; license file coming.
