# Functional Requirements

| ID              | Requirement |
| --------------- | ----------- |
| FR1             | Register/manage monitored sites: domain, display name, check interval, expected content selector |
| FR2             | Multi-stage check pipeline per site: DNS resolution (hand-rolled UDP query/parse), TCP connect, TLS handshake + cert expiry (tokio-rustls), HTTP request (hand-rolled request/response over the established connection, TTFB + full response), content assertion — each stage timed independently |
| FR3             | Persist every check result with per-stage timings, status, and success/failure — partial results (e.g. DNS + TCP succeeded, TLS failed) are valid rows, not errors |
| FR4             | Retry-before-alert failure logic — N consecutive failures within a time window before an incident is opened, not a single dropped check |
| FR5             | Multi-region checking (2-3 AWS regions), correlated via a shared `check_round_id` so results from independent regional workers can be joined back into one logical check event |
| FR6             | Content-change detection - hash/diff of key page content between checks, flagged as a distinct event from downtime |
| FR7             | Alerting on confirmed incidents (email and/or webhook) |
| FR8             | Incident history/timeline per site (start, resolution, cause, duration) |
| FR9             | Live dashboard: real-time status grid, response-time graphs per site, per-check waterfall view |
| FR10 (stretch)  | Anomaly detection - flag "degraded" when response time deviates from a site's own rolling baseline |
| FR11 (stretch)  | Synthetic multi-step checks (e.g. simulate a login flow, not just load a homepage) |
| FR12 (stretch)  | Public status-page generator per monitored site |
