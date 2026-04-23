CREATE TABLE share_views (
    share_hash TEXT NOT NULL,
    session_id TEXT NOT NULL,
    viewed_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (share_hash, session_id)
);
