-- Add migration script here
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE TABLE IF NOT EXISTS url (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain                  TEXT NOT NULL,
    name                    TEXT NOT NULL,
    check_interval_seconds  INT NOT NULL DEFAULT 60,
    expected_content        TEXT,
    is_active               BOOLEAN NOT NULL DEFAULT true,
    next_check_at           TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at              TIMESTAMPTZ NOT NULL DEFAULT now()
);
