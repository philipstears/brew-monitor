use bm_tilt::TiltColor;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Connection, Result, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Serialize, Deserialize)]
pub struct TiltReading {
    pub at: DateTime<Utc>,
    pub fahrenheit: u16,
    pub gravity: u16,
}

#[derive(Clone)]
pub struct TiltData {
    color: String,
    connection: Arc<Mutex<Connection>>,
}

impl TiltData {
    pub(super) fn new(connection: Arc<Mutex<Connection>>, color: TiltColor) -> Self {
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
        let mut statement =
            connection.prepare("select at,temp,grav from tilt_readings where at >= ? and at < ? order by at asc")?;

        let readings = statement
            .query_map(params![from.timestamp(), to_excl.timestamp()], |row| {
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
        self.connection
            .lock()
            .unwrap_or_else(|_| unreachable!("The connection mutex has been poisoned, this should not be possible"))
    }
}
