-- Add migration script here
CREATE TABLE IF NOT EXISTS content_snapshots (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    url_id              UUID NOT NULL REFERENCES url(id) ON DELETE CASCADE,
    content_hash        TEXT NOT NULL,
    captured_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_content_snapshots_site_time
    ON content_snapshots (url_id, captured_at DESC);
