use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, OptionalExtension, Result};
use serde::{Deserialize, Serialize};

use super::WrappedConnection;

#[derive(Serialize, Deserialize)]
pub struct DHT22Reading {
    at: DateTime<Utc>,
    temp: u16,
    humidity: u16,
}

#[derive(Serialize, Deserialize)]
pub struct DHT22Info {
    alias: String,
    pin: u8,
    enabled: bool,
}

#[derive(Clone)]
pub struct DHT22Data(WrappedConnection);

impl DHT22Data {
    pub(super) fn new(connection: WrappedConnection) -> Self {
        Self(connection)
    }

    pub fn try_get_info(&self, alias: &str) -> Result<Option<DHT22Info>> {
        let connection_guard = self.0.lock_or_panic();
        let mut statement = connection_guard.prepare("select pin,enabled from dht22_devices where alias = ?")?;
        statement
            .query_row(params![alias], |row| {
                Ok(DHT22Info {
                    alias: alias.into(),
                    pin: row.get(0)?,
                    enabled: row.get(1)?,
                })
            })
            .optional()
    }

    pub fn upsert(&self, info: &DHT22Info) -> Result<()> {
        let connection = self.0.lock_or_panic();

        connection.execute(
            "
            insert into dht22_devices (alias,pin,enabled)
            values (?,?,?)
            on conflict(alias) do update set pin=excluded.pin,enabled=excluded.enabled;
            ",
            params![info.alias, info.pin, info.enabled],
        )?;

        Ok(())
    }

    pub fn new_readings_inserter(&self, alias: &str) -> Result<Option<DHT22ReadingsInserter>> {
        let id = {
            let connection_guard = self.0.lock_or_panic();
            let mut statement = connection_guard.prepare("select id from dht22_devices where alias = ?")?;
            statement.query_row(params![alias], |row| Ok(row.get(0)?))
        };

        id.map(|id| DHT22ReadingsInserter::new(self.0.clone(), id)).optional()
    }

    pub fn get_readings(&self, alias: &str, from: DateTime<Utc>, to_excl: DateTime<Utc>) -> Result<Vec<DHT22Reading>> {
        let connection = self.0.lock_or_panic();

        let mut statement = connection.prepare(
            "
            select at,temp,humidity from dht22_readings
            inner join dht22_devices
            on dht22_devices.id = dht22_readings.id
            where dht22_devices.alias = ?
            and at >= ?
            and at < ?
            order by at asc
            ",
        )?;

        let readings = statement
            .query_map(params![alias, from.timestamp(), to_excl.timestamp()], |row| {
                Ok(DHT22Reading {
                    at: Utc.timestamp(row.get(0)?, 0),
                    temp: row.get(1)?,
                    humidity: row.get(2)?,
                })
            })?
            .collect();

        readings
    }
}

pub struct DHT22ReadingsInserter {
    connection: WrappedConnection,
    id: i64,
}

impl DHT22ReadingsInserter {
    fn new(connection: WrappedConnection, id: i64) -> Self {
        Self {
            connection,
            id,
        }
    }

    pub fn insert(&self, temperature: u16, humidity: u16) -> Result<()> {
        let at = chrono::Utc::now().naive_utc();

        self.connection.lock_or_panic().execute(
            "insert into dht22_readings (id, at, temp, humidity) values (?1, ?2, ?3, ?4)",
            params![self.id, at.timestamp(), temperature, humidity],
        )?;

        Ok(())
    }
}
