use dht22_pi as dht22;
use chrono::prelude::*;

const PIN: u8 = 4;

fn main() {
    loop {
        let result = dht22::read(PIN);
        let now = Utc::now();

        println!("{:?}: {:?}", now, result);

        std::thread::sleep(std::time::Duration::from_secs(5));
    }
}
