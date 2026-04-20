-- Allow setup slots to be NULL ("Default" placeholder — part missing from inventory)
ALTER TABLE setups
    ALTER COLUMN engine_id     DROP NOT NULL,
    ALTER COLUMN front_wing_id DROP NOT NULL,
    ALTER COLUMN rear_wing_id  DROP NOT NULL,
    ALTER COLUMN suspension_id DROP NOT NULL,
    ALTER COLUMN brakes_id     DROP NOT NULL,
    ALTER COLUMN gearbox_id    DROP NOT NULL;
