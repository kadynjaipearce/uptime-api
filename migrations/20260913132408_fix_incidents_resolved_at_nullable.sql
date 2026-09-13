-- Add migration script here
ALTER TABLE incidents
    ALTER COLUMN resolved_at DROP NOT NULL;
