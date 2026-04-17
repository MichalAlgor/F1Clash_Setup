-- ── inventory ────────────────────────────────────────────────────────────────
ALTER TABLE inventory ADD COLUMN IF NOT EXISTS session_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS inventory_session_idx ON inventory(session_id);

-- ── driver_inventory ─────────────────────────────────────────────────────────
ALTER TABLE driver_inventory ADD COLUMN IF NOT EXISTS session_id TEXT NOT NULL DEFAULT '';
ALTER TABLE driver_inventory
    DROP CONSTRAINT IF EXISTS driver_inventory_season_key,
    ADD CONSTRAINT driver_inventory_session_key UNIQUE (session_id, driver_name, rarity, season);
CREATE INDEX IF NOT EXISTS driver_inventory_session_idx ON driver_inventory(session_id);

-- ── boosts ───────────────────────────────────────────────────────────────────
ALTER TABLE boosts ADD COLUMN IF NOT EXISTS session_id TEXT NOT NULL DEFAULT '';
ALTER TABLE boosts
    DROP CONSTRAINT IF EXISTS boosts_season_key,
    ADD CONSTRAINT boosts_session_key UNIQUE (session_id, part_name, season);
CREATE INDEX IF NOT EXISTS boosts_session_idx ON boosts(session_id);

-- ── driver_boosts ────────────────────────────────────────────────────────────
ALTER TABLE driver_boosts ADD COLUMN IF NOT EXISTS session_id TEXT NOT NULL DEFAULT '';
ALTER TABLE driver_boosts
    DROP CONSTRAINT IF EXISTS driver_boosts_season_key,
    ADD CONSTRAINT driver_boosts_session_key UNIQUE (session_id, driver_name, rarity, season);
CREATE INDEX IF NOT EXISTS driver_boosts_session_idx ON driver_boosts(session_id);

-- ── setups ───────────────────────────────────────────────────────────────────
ALTER TABLE setups ADD COLUMN IF NOT EXISTS session_id TEXT NOT NULL DEFAULT '';
CREATE INDEX IF NOT EXISTS setups_session_idx ON setups(session_id);

-- ── settings ─────────────────────────────────────────────────────────────────
-- Change PK from (key) to (key, session_id) so each session has its own active_season.
ALTER TABLE settings ADD COLUMN IF NOT EXISTS session_id TEXT NOT NULL DEFAULT '';
ALTER TABLE settings DROP CONSTRAINT IF EXISTS settings_pkey;
ALTER TABLE settings ADD PRIMARY KEY (key, session_id);
-- Existing row ('active_season', '2025') keeps session_id = '' — the owner's data.
