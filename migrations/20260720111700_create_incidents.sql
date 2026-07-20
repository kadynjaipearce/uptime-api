-- Add migration script here
CREATE TABLE IF NOT EXISTS incidents (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url_id              UUID NOT NULL REFERENCES url(id) ON DELETE CASCADE,
    started_at          TIMESTAMPTZ NOT NULL,
    resolved_at         TIMESTAMPTZ NOT NULL,
    resolved            BOOLEAN NOT NULL DEFAULT false,
    cause               TEXT
);

CREATE INDEX idx_incidents_site_open
    ON incidents (url_id, resolved);
