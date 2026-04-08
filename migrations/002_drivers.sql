CREATE TABLE driver_inventory (
    id SERIAL PRIMARY KEY,
    driver_name TEXT NOT NULL,
    rarity TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    UNIQUE (driver_name, rarity)
);

CREATE TABLE driver_boosts (
    id SERIAL PRIMARY KEY,
    driver_name TEXT NOT NULL,
    rarity TEXT NOT NULL,
    percentage INTEGER NOT NULL DEFAULT 0,
    UNIQUE (driver_name, rarity)
);

ALTER TABLE setups
    ADD COLUMN driver1_id INTEGER REFERENCES driver_inventory(id),
    ADD COLUMN driver2_id INTEGER REFERENCES driver_inventory(id);
