CREATE TYPE part_category AS ENUM (
    'engine',
    'front_wing',
    'rear_wing',
    'sidepod',
    'underbody',
    'suspension',
    'brakes'
);

CREATE TABLE parts (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    category part_category NOT NULL,
    level INTEGER NOT NULL DEFAULT 1,
    speed INTEGER NOT NULL DEFAULT 0,
    cornering INTEGER NOT NULL DEFAULT 0,
    power_unit INTEGER NOT NULL DEFAULT 0,
    qualifying INTEGER NOT NULL DEFAULT 0,
    pit_stop_time DOUBLE PRECISION NOT NULL DEFAULT 0.0
);

CREATE TABLE setups (
    id SERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    engine_id INTEGER NOT NULL REFERENCES parts(id),
    front_wing_id INTEGER NOT NULL REFERENCES parts(id),
    rear_wing_id INTEGER NOT NULL REFERENCES parts(id),
    sidepod_id INTEGER NOT NULL REFERENCES parts(id),
    underbody_id INTEGER NOT NULL REFERENCES parts(id),
    suspension_id INTEGER NOT NULL REFERENCES parts(id),
    brakes_id INTEGER NOT NULL REFERENCES parts(id)
);
