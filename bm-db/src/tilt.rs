use bm_tilt::TiltColor;
use rusqlite::{params, Connection, Result, NO_PARAMS};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Serialize, Deserialize)]
pub struct TiltReading {
    pub at: String,
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

    pub(super) fn create_table(connection: &Connection) -> Result<()> {
        connection.execute(
            "create table if not exists tilt_readings (
                 \"when\" text primary key,
                 colour text not null,
                 temperature integer not null,
                 gravity integer not null
             )",
            NO_PARAMS,
        )?;

        Ok(())
    }

    pub fn insert_reading(&self, fahrenheit: u16, gravity: u16) -> Result<()> {
        let when = chrono::Utc::now().naive_utc();

        self.connection().execute(
            "INSERT INTO tilt_readings (\"when\", colour, temperature, gravity) values (?1, ?2, ?3, ?4)",
            params![when, self.color, fahrenheit, gravity],
        )?;

        Ok(())
    }

    fn connection(&self) -> MutexGuard<Connection> {
        self.connection
            .lock()
            .unwrap_or_else(|_| unreachable!("The connection mutex has been poisoned, this should not be possible"))
    }
}
