use bm_tilt::TiltColor;
use chrono::{DateTime, TimeZone, Utc};
use rusqlite::{params, Result};
use serde::{Deserialize, Serialize};

use super::WrappedConnection;

#[derive(Serialize, Deserialize)]
pub struct TiltReading {
    pub at: DateTime<Utc>,
    pub fahrenheit: u16,
    pub gravity: u16,
}

#[derive(Clone)]
pub struct TiltData(WrappedConnection);

impl TiltData {
    pub(super) fn new(connection: WrappedConnection) -> Self {
        Self(connection)
    }

    pub fn new_readings_inserter(&self, color: &TiltColor) -> TiltReadingsInserter {
        TiltReadingsInserter::new(self.0.clone(), color.to_string())
    }

    pub fn get_readings(
        &self,
        color: &TiltColor,
        from: DateTime<Utc>,
        to_excl: DateTime<Utc>,
    ) -> Result<Vec<TiltReading>> {
        let connection = self.0.lock_or_panic();
        let mut statement = connection
            .prepare("select at,temp,grav from tilt_readings where which = ? and at >= ? and at < ? order by at asc")?;

        let readings = statement
            .query_map(params![color.to_string(), from.timestamp(), to_excl.timestamp()], |row| {
                Ok(TiltReading {
                    at: Utc.timestamp(row.get(0)?, 0),
                    fahrenheit: row.get(1)?,
                    gravity: row.get(2)?,
                })
            })?
            .collect();

        readings
    }
}

pub struct TiltReadingsInserter {
    connection: WrappedConnection,
    which: String,
}

impl TiltReadingsInserter {
    fn new(connection: WrappedConnection, which: String) -> Self {
        Self {
            connection,
            which,
        }
    }

    pub fn insert(&self, fahrenheit: u16, gravity: u16) -> Result<()> {
        let at = chrono::Utc::now().naive_utc();

        self.connection.lock_or_panic().execute(
            "insert into tilt_readings (at, which, temp, grav) values (?1, ?2, ?3, ?4)",
            params![at, self.which, fahrenheit, gravity],
        )?;

        Ok(())
    }
}
