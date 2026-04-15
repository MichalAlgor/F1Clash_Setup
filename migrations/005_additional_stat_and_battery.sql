-- 1. Battery part category
ALTER TYPE part_category ADD VALUE IF NOT EXISTS 'battery';

-- 2. Additional stat name on part definitions (nullable — null means no special stat)
ALTER TABLE part_catalog
    ADD COLUMN IF NOT EXISTS additional_stat_name TEXT DEFAULT NULL;

-- 3. Additional stat columns on level stats (replaces drs)
ALTER TABLE part_level_stats
    ADD COLUMN IF NOT EXISTS additional_stat_value   INTEGER NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS additional_stat_details JSONB   NOT NULL DEFAULT '{}';

-- 4. Migrate existing DRS data into the new generic columns
UPDATE part_catalog
    SET additional_stat_name = 'DRS'
    WHERE category = 'rear_wing'
      AND EXISTS (
          SELECT 1 FROM part_level_stats pls
          WHERE pls.part_id = part_catalog.id AND pls.drs > 0
      );

UPDATE part_level_stats
    SET additional_stat_value = drs
    WHERE drs > 0;

ALTER TABLE part_level_stats DROP COLUMN IF EXISTS drs;

-- 5. Season categories — which part slots each season uses
CREATE TABLE IF NOT EXISTS season_categories (
    season   TEXT          NOT NULL,
    category part_category NOT NULL,
    UNIQUE (season, category)
);

-- Seed 2025 with the original 6 categories
INSERT INTO season_categories (season, category) VALUES
    ('2025', 'brakes'),
    ('2025', 'gearbox'),
    ('2025', 'rear_wing'),
    ('2025', 'front_wing'),
    ('2025', 'suspension'),
    ('2025', 'engine')
ON CONFLICT DO NOTHING;

-- 6. Battery slot in setups (nullable — only populated in seasons that include Battery)
ALTER TABLE setups ADD COLUMN IF NOT EXISTS battery_id INTEGER REFERENCES inventory(id);
