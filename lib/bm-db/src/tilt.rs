use bm_tilt::TiltColor;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::sync::MutexGuard;

use super::WrappedConnection;

#[derive(Serialize, Deserialize)]
pub struct TiltReading {
    pub at: DateTime<Utc>,
    pub fahrenheit: u16,
    pub gravity: u16,
}

#[derive(Clone)]
pub struct TiltData {
    color: String,
    connection: WrappedConnection,
}

impl TiltData {
    pub(super) fn new(connection: WrappedConnection, color: TiltColor) -> Self {
        Self {
            color: color.to_string(),
            connection,
        }
    }

    pub fn insert_reading(&self, fahrenheit: u16, gravity: u16) -> Result<()> {
        let at = Utc::now().timestamp();

        self.connection().execute(
            "INSERT INTO tilt_readings (at, which, temp, grav) values (?1, ?2, ?3, ?4)",
            params![at, self.color, fahrenheit, gravity],
        )?;

        Ok(())
    }

    pub fn get_readings(&self, from: DateTime<Utc>, to_excl: DateTime<Utc>) -> Result<Vec<TiltReading>> {
        let connection = self.connection();
        let mut statement = connection
            .prepare("select at,temp,grav from tilt_readings where which = ? and at >= ? and at < ? order by at asc")?;

        let readings = statement
            .query_map(params![&self.color, from.timestamp(), to_excl.timestamp()], |row| {
                Ok(TiltReading {
                    at: Utc.timestamp(row.get(0)?, 0),
                    fahrenheit: row.get(1)?,
                    gravity: row.get(2)?,
                })
            })?
            .collect();

        readings
    }

    fn connection(&self) -> MutexGuard<Connection> {
        self.connection.lock_or_panic()
    }
}
