CREATE TABLE shared_setups (
    id SERIAL PRIMARY KEY,
    share_hash TEXT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    season TEXT NOT NULL,
    priorities JSONB NOT NULL,
    parts_snapshot JSONB NOT NULL,
    drivers_snapshot JSONB NOT NULL,
    total_parts JSONB NOT NULL,
    total_drivers JSONB NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX idx_shared_setups_hash ON shared_setups(share_hash);
CREATE INDEX idx_shared_setups_created ON shared_setups(created_at DESC);
