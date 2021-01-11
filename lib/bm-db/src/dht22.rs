use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, OptionalExtension, Result};
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;

use super::WrappedConnection;

#[derive(Clone)]
pub struct DHT22Data {
    id: i64,
    pin: u8,
    connection: WrappedConnection,
}

#[derive(Serialize, Deserialize)]
pub struct DHT22Reading {
    at: DateTime<Utc>,
    temp: u16,
    humidity: u16,
}

impl DHT22Data {
    pub(super) fn try_get(connection: WrappedConnection, name: &str) -> Result<Option<Self>> {
        let result = {
            let connection_guard = connection.lock_or_panic();
            let mut statement = connection_guard.prepare("select id,pin from dht22_devices where alias = ?")?;

            statement.query_row(params![name], |row| {
                let id = row.get(0)?;
                let pin = row.get(1)?;
                Ok((id, pin))
            })
        };

        result
            .map(|(id, pin)| Self {
                connection,
                id,
                pin,
            })
            .optional()
    }

    pub fn insert_reading(&self, temperature: u16, humidity: u16) -> Result<()> {
        let when = chrono::Utc::now().naive_utc();

        self.connection().execute(
            "INSERT INTO dht22_readings (id, at, temp, humidity) values (?1, ?2, ?3, ?4)",
            params![self.id, when, temperature, humidity],
        )?;

        Ok(())
    }

    pub fn get_readings(&self, from: DateTime<Utc>, to_excl: DateTime<Utc>) -> Result<Vec<DHT22Reading>> {
        let connection = self.connection();

        let mut statement = connection.prepare(
            "select at,temp,humidity from dht22_readings where id = ? and at >= ? and at < ? order by at asc",
        )?;

        let readings = statement
            .query_map(params![self.id, from.timestamp(), to_excl.timestamp()], |row| {
                Ok(DHT22Reading {
                    at: Utc.timestamp(row.get(0)?, 0),
                    temp: row.get(1)?,
                    humidity: row.get(2)?,
                })
            })?
            .collect();

        readings
    }

    fn connection(&self) -> MutexGuard<Connection> {
        self.connection.lock_or_panic()
    }
}
