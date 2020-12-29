use bm_tilt::TiltColor;
use rusqlite::{Connection, Result};
use std::sync::{Arc, Mutex};

mod tilt;
pub use tilt::*;

mod dht22;
pub use dht22::*;

#[derive(Clone)]
pub struct DB {
    connection: Arc<Mutex<Connection>>,
}

impl DB {
    pub fn open(path: &str) -> Result<Self> {
        let connection = Connection::open(path)?;

        TiltData::create_table(&connection)?;
        DHT22Data::create_table(&connection)?;

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
}
