use rusqlite::{params, Connection, Result, NO_PARAMS};
use std::sync::{Arc, Mutex, MutexGuard};

#[derive(Clone)]
pub struct DHT22Data {
    name: String,
    connection: Arc<Mutex<Connection>>,
}

impl DHT22Data {
    pub(super) fn new(connection: Arc<Mutex<Connection>>, name: String) -> Self {
        Self {
            name,
            connection,
        }
    }

    pub(super) fn create_table(connection: &Connection) -> Result<()> {
        connection.execute(
            "create table if not exists dht22_readings (
                 \"when\" text primary key,
                 which text not null,
                 temperature integer not null,
                 humidity integer not null
             )",
            NO_PARAMS,
        )?;

        Ok(())
    }

    pub fn insert_reading(&self, temperature: u16, humidity: u16) -> Result<()> {
        let when = chrono::Utc::now().naive_utc();

        self.connection().execute(
            "INSERT INTO dht22_readings (\"when\", which, temperature, humidity) values (?1, ?2, ?3, ?4)",
            params![when, self.name, temperature, humidity],
        )?;

        Ok(())
    }

    fn connection(&self) -> MutexGuard<Connection> {
        self.connection
            .lock()
            .unwrap_or_else(|_| unreachable!("The connection mutex has been poisoned, this should not be possible"))
    }
}
