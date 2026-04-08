CREATE TYPE part_category AS ENUM (
    'engine',
    'front_wing',
    'rear_wing',
    'suspension',
    'brakes',
    'gearbox'
);

CREATE TABLE inventory (
    id SERIAL PRIMARY KEY,
    part_name TEXT NOT NULL,
    level INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE setups (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    engine_id INTEGER NOT NULL REFERENCES inventory(id),
    front_wing_id INTEGER NOT NULL REFERENCES inventory(id),
    rear_wing_id INTEGER NOT NULL REFERENCES inventory(id),
    suspension_id INTEGER NOT NULL REFERENCES inventory(id),
    brakes_id INTEGER NOT NULL REFERENCES inventory(id),
    gearbox_id INTEGER NOT NULL REFERENCES inventory(id)
);
