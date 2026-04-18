ALTER TABLE page_events ADD COLUMN session_id TEXT;

CREATE INDEX IF NOT EXISTS page_events_session_idx ON page_events (session_id);
