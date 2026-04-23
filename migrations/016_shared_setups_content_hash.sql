ALTER TABLE shared_setups ADD COLUMN content_hash TEXT;

-- Partial unique index: only enforces uniqueness for non-NULL values,
-- so existing rows (which have no content_hash) never conflict.
CREATE UNIQUE INDEX idx_shared_setups_content_hash
    ON shared_setups(content_hash)
    WHERE content_hash IS NOT NULL;
