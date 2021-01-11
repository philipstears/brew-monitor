use bm_tilt::TiltColor;
use rusqlite::Connection;
use std::sync::{Arc, Mutex, MutexGuard};

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
    connection: WrappedConnection,
}

impl DB {
    pub fn open(path: &str) -> Result<Self, OpenError> {
        let connection = Connection::open(path)?;

        Self::upgrade_db(&connection)?;

        let result = Self {
            connection: WrappedConnection::new(connection),
        };

        Ok(result)
    }

    pub fn tilt_ensure(&self, color: &TiltColor) -> TiltData {
        TiltData::new(self.connection.clone(), *color)
    }

    pub fn dht22_try_get(&self, name: &str) -> Result<Option<DHT22Data>, rusqlite::Error> {
        DHT22Data::try_get(self.connection.clone(), name)
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

#[derive(Clone)]
struct WrappedConnection(Arc<Mutex<Connection>>);

impl WrappedConnection {
    fn new(connection: Connection) -> Self {
        Self(Arc::new(Mutex::new(connection)))
    }

    pub fn lock_or_panic(&self) -> MutexGuard<Connection> {
        self.0
            .lock()
            .unwrap_or_else(|_| unreachable!("The connection mutex has been poisoned, this should not be possible"))
    }
}
