-- Driver definitions table (season-scoped, like part_catalog)
CREATE TABLE driver_catalog (
    id         SERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    season     TEXT NOT NULL,
    rarity     TEXT NOT NULL,
    series     TEXT NOT NULL DEFAULT '1',
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE (name, rarity, season)
);

-- Per-level stats for each driver definition
CREATE TABLE driver_level_stats (
    id              SERIAL PRIMARY KEY,
    driver_id       INTEGER NOT NULL REFERENCES driver_catalog(id) ON DELETE CASCADE,
    level           INTEGER NOT NULL,
    overtaking      INTEGER NOT NULL,
    defending       INTEGER NOT NULL,
    qualifying      INTEGER NOT NULL,
    race_start      INTEGER NOT NULL,
    tyre_management INTEGER NOT NULL,
    cards_required  INTEGER NOT NULL DEFAULT 0,
    coins_cost      BIGINT NOT NULL DEFAULT 0,
    legacy_points   INTEGER NOT NULL DEFAULT 0,
    UNIQUE (driver_id, level)
);

CREATE INDEX driver_catalog_season_idx ON driver_catalog(season);
