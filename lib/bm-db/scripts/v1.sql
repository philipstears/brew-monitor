create table if not exists dht22_readings (
    "when" text primary key,
    which text not null,
    temperature integer not null,
    humidity integer not null
    );

create table if not exists tilt_readings (
    "when" text primary key,
    colour text not null,
    temperature integer not null,
    gravity integer not null
    );

-- -----------------------------------------------------------------------------
-- Meta
-- -----------------------------------------------------------------------------
pragma user_version=1;
