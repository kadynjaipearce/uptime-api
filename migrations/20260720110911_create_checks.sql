-- Add migration script here
CREATE EXTENSION IF NOT EXISTS timescaledb;

CREATE TABLE IF NOT EXISTS checks (
    id                  UUID NOT NULL DEFAULT gen_random_uuid(),
    url_id              UUID NOT NULL REFERENCES url(id) ON DELETE CASCADE,
    check_round_id      UUID NOT NULL,
    region              TEXT NOT NULL,
    checked_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    dns_ms              INT,
    connect_ms          INT,
    tls_ms              INT,
    ttfb_ms             INT,
    total_ms            INT,

    status_code         INT,
    success             BOOLEAN NOT NULL,
    error_stage         TEXT,
    error_message       TEXT,
    content_hash        TEXT,

    PRIMARY KEY (id, checked_at)
);

SELECT create_hypertable('checks', 'checked_at');

CREATE INDEX idx_checks_site_time
    ON checks (url_id, checked_at DESC);

CREATE INDEX idx_checks_round
    ON checks (check_round_id);
