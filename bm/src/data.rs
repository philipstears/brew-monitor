use rusqlite::{params, Connection, Result, NO_PARAMS};
use std::sync::{Arc, Mutex};

use bm_tilt::Tilt;

#[derive(Clone)]
pub struct DB {
    connection: Arc<Mutex<Connection>>,
}

impl DB {
    pub fn open(path: &str) -> Result<Self> {
        let connection = Connection::open(path)?;

        connection.execute(
            "create table if not exists tilt_readings (
                 \"when\" text primary key,
                 colour text not null,
                 temperature integer not null,
                 gravity integer not null
             )",
            NO_PARAMS,
        )?;

        connection.execute(
            "create table if not exists dht22_readings (
                 \"when\" text primary key,
                 which text not null,
                 temperature integer not null,
                 humidity integer not null
             )",
            NO_PARAMS,
        )?;

        let result = Self {
            connection: Arc::new(Mutex::new(connection)),
        };

        Ok(result)
    }

    pub fn insert_tilt_reading(&self, tilt: &Tilt) {
        let connection = self.connection.lock().unwrap();
        let when = chrono::Utc::now().naive_utc();

        connection
            .execute(
                "INSERT INTO tilt_readings (\"when\", colour, temperature, gravity) values (?1, ?2, ?3, ?4)",
                params![when, tilt.color.to_string(), tilt.fahrenheit, tilt.gravity],
            )
            .unwrap();
    }

    pub fn insert_dht22_reading(&self, which: String, temperature: u16, gravity: u16) {
        let connection = self.connection.lock().unwrap();
        let when = chrono::Utc::now().naive_utc();

        connection
            .execute(
                "INSERT INTO tilt_readings (\"when\", which, temperature, gravity) values (?1, ?2, ?3, ?4)",
                params![when, which, temperature, gravity],
            )
            .unwrap();
    }
}
