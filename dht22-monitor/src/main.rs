use chrono::prelude::*;
use dht22_pi as dht22;

const PIN: u8 = 4;

fn main() {
    loop {
        let now = Utc::now();

        match dht22::read(PIN) {
            Ok(dht22::Reading {
                temperature,
                humidity,
            }) => {
                println!("at={:?} celsius={:?} humidity={:?}", now, temperature, humidity);
            }
            Err(err) => {
                eprintln!("at={:?} error={:?}", now, err);
            }
        }

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
