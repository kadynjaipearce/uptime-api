-- Add migration script here
CREATE TABLE IF NOT EXISTS jobs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    job_type            TEXT NOT NULL,
    payload             JSONB NOT NULL,
    status              TEXT NOT NULL,
    attempts            INT NOT NULL DEFAULT 0,
    max_attempts        INT NOT NULL DEFAULT 3,
    run_at              TIMESTAMPTZ NOT NULL DEFAULT now(),
    claimed_by          TEXT,
    claimed_at          TIMESTAMPTZ,
    error_message       TEXT,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_jobs_claimable
    ON jobs (status, run_at, (payload->>'region'))
    WHERE status = 'pending';
