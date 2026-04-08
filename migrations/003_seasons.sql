CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
INSERT INTO settings (key, value) VALUES ('active_season', '2025');

ALTER TABLE inventory ADD COLUMN season TEXT NOT NULL DEFAULT '2025';
ALTER TABLE driver_inventory ADD COLUMN season TEXT NOT NULL DEFAULT '2025';
ALTER TABLE boosts ADD COLUMN season TEXT NOT NULL DEFAULT '2025';
ALTER TABLE driver_boosts ADD COLUMN season TEXT NOT NULL DEFAULT '2025';
ALTER TABLE setups ADD COLUMN season TEXT NOT NULL DEFAULT '2025';

ALTER TABLE driver_inventory DROP CONSTRAINT driver_inventory_driver_name_rarity_key;
ALTER TABLE driver_inventory ADD CONSTRAINT driver_inventory_season_key UNIQUE (driver_name, rarity, season);

ALTER TABLE driver_boosts DROP CONSTRAINT driver_boosts_driver_name_rarity_key;
ALTER TABLE driver_boosts ADD CONSTRAINT driver_boosts_season_key UNIQUE (driver_name, rarity, season);

ALTER TABLE boosts DROP CONSTRAINT boosts_part_name_key;
ALTER TABLE boosts ADD CONSTRAINT boosts_season_key UNIQUE (part_name, season);
