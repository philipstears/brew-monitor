begin exclusive transaction;

    -- -----------------------------------------------------------------------------
    -- DHT22 Date Bug Fix
    -- -----------------------------------------------------------------------------
    alter table dht22_readings rename to dht22_readings_old;

    create table dht22_readings (
        id integer not null,
        at integer not null,
        temp integer not null,
        humidity integer not null,
        foreign key(id) references dht22_devices(id)
        );

    insert into dht22_readings(at, id, temp, humidity)
    select coalesce(strftime('%s', at), at), id, temp, humidity
    from dht22_readings_old;

    drop table dht22_readings_old;

    create index idx_dht22_readings_id
    on dht22_readings (id);

    create index idx_dht22_readings_at
    on dht22_readings (at);

    -- -----------------------------------------------------------------------------
    -- Tilt Date Bug Fix
    -- -----------------------------------------------------------------------------
    alter table tilt_readings rename to tilt_readings_old;

    create table tilt_readings (
        at integer not null,
        which text not null,
        temp integer not null,
        grav integer not null
        );

    insert into tilt_readings(at, which, temp, grav)
    select coalesce(strftime('%s', at), at), which, temp, grav
    from tilt_readings_old;

    drop table tilt_readings_old;

    create index idx_tilt_readings_which
    on tilt_readings (which);

    create index idx_tilt_readings_at
    on tilt_readings (at);

    -- -----------------------------------------------------------------------------
    -- Meta
    -- -----------------------------------------------------------------------------
    pragma user_version=3;

commit transaction;
