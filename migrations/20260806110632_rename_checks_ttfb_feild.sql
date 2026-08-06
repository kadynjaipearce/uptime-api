-- Add migration script here
ALTER TABLE checks RENAME COLUMN ttfb_ms TO http_response_time_ms;
