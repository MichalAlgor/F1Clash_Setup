-- Structured behavioral signals recorded from route handlers.
-- Fully anonymous: session_id is the same SHA-256 hash used in page_events.
-- properties JSONB holds only categorical/bucketed data — never PII.
CREATE TABLE feature_events (
    id         BIGSERIAL PRIMARY KEY,
    session_id TEXT        NOT NULL,
    event      TEXT        NOT NULL,
    properties JSONB       NOT NULL DEFAULT '{}',
    ts         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX feature_events_ts_idx      ON feature_events (ts);
CREATE INDEX feature_events_event_idx   ON feature_events (event, ts);
CREATE INDEX feature_events_session_idx ON feature_events (session_id, ts);
