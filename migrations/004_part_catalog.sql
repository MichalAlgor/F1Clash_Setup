CREATE TABLE part_catalog (
    id         SERIAL PRIMARY KEY,
    name       TEXT NOT NULL,
    season     TEXT NOT NULL,
    category   part_category NOT NULL,
    series     INTEGER NOT NULL,
    rarity     TEXT NOT NULL CHECK (rarity IN ('Common', 'Rare', 'Epic')),
    sort_order INTEGER NOT NULL DEFAULT 0,
    UNIQUE (name, season)
);

CREATE TABLE part_level_stats (
    id            SERIAL PRIMARY KEY,
    part_id       INTEGER NOT NULL REFERENCES part_catalog(id) ON DELETE CASCADE,
    level         INTEGER NOT NULL,
    speed         INTEGER NOT NULL,
    cornering     INTEGER NOT NULL,
    power_unit    INTEGER NOT NULL,
    qualifying    INTEGER NOT NULL,
    pit_stop_time DOUBLE PRECISION NOT NULL,
    drs           INTEGER NOT NULL DEFAULT 0,
    UNIQUE (part_id, level)
);

CREATE INDEX part_catalog_season_idx ON part_catalog(season);
