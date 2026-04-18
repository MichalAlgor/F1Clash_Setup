CREATE TABLE IF NOT EXISTS page_events (
    id           BIGSERIAL PRIMARY KEY,
    path         TEXT        NOT NULL,
    method       TEXT        NOT NULL,
    status       SMALLINT    NOT NULL,
    referrer     TEXT,
    device       TEXT        NOT NULL,
    country      CHAR(2),
    response_ms  INTEGER     NOT NULL,
    ts           TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Supports time-range queries and pruning
CREATE INDEX IF NOT EXISTS page_events_ts_idx ON page_events (ts DESC);

-- Supports "top paths" queries
CREATE INDEX IF NOT EXISTS page_events_ts_path_idx ON page_events (ts DESC, path);
