use bm_tilt::TiltColor;
use rusqlite::{Connection, NO_PARAMS};
use std::sync::{Arc, Mutex};

mod tilt;
pub use tilt::*;

mod dht22;
pub use dht22::*;

const V1: &'static str = include_str!("../scripts/v1.sql");
const V2: &'static str = include_str!("../scripts/v2.sql");

#[derive(Debug)]
pub enum OpenError {
    SQLiteError(rusqlite::Error),
    UnexpectedVersion(u32),
}

impl From<rusqlite::Error> for OpenError {
    fn from(other: rusqlite::Error) -> Self {
        Self::SQLiteError(other)
    }
}

enum Version {
    Uninitialized,
    Alpha1,
    Alpha2,
}

#[derive(Clone)]
pub struct DB {
    connection: Arc<Mutex<Connection>>,
}

impl DB {
    pub fn open(path: &str) -> Result<Self, OpenError> {
        let connection = Connection::open(path)?;

        Self::upgrade_db(&connection)?;

        let result = Self {
            connection: Arc::new(Mutex::new(connection)),
        };

        Ok(result)
    }

    pub fn get_tilt(&self, color: &TiltColor) -> TiltData {
        TiltData::new(self.connection.clone(), *color)
    }

    pub fn get_dht22(&self, name: String) -> DHT22Data {
        DHT22Data::new(self.connection.clone(), name)
    }

    /// Recursively updates the database version to the latest.
    fn upgrade_db(connection: &Connection) -> Result<(), OpenError> {
        match Self::get_db_version(&connection)? {
            Version::Uninitialized => {
                connection.execute_batch(V1)?;
                return Self::upgrade_db(connection);
            }

            Version::Alpha1 => {
                connection.execute_batch(V2)?;
                return Self::upgrade_db(connection);
            }

            Version::Alpha2 => {
                return Ok(());
            }
        }
    }

    fn get_db_version(connection: &Connection) -> Result<Version, OpenError> {
        let raw_version = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;

        match raw_version {
            0 => Ok(Version::Uninitialized),
            1 => Ok(Version::Alpha1),
            2 => Ok(Version::Alpha2),
            n => Err(OpenError::UnexpectedVersion(n)),
        }
    }
}
