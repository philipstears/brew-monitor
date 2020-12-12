mod bluetooth_discovery;
pub use bluetooth_discovery::*;

mod devices;
use devices::gf_manager::GrainfatherManager;

mod data;
use data::DB;

mod web;

use bm_tilt::*;
use chrono::prelude::*;
use dht22_pi as dht22;
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, RwLock},
};
use warp::Filter;

const PIN: u8 = 4;

#[tokio::main]
pub async fn main() {
    pretty_env_logger::init();

    let db = DB::open("brew-monitor.db").unwrap();
    let tilts = Arc::new(RwLock::new(HashMap::<TiltColor, DeviceInfo<Tilt>>::new()));
    let gf = GrainfatherManager::new();

    let routes = {
        let web_content = web::assets::route();
        let gf_route = web::gf::route(gf.clone());
        let tilt_route = web::tilt::route(tilts.clone());
        web_content.or(tilt_route).or(gf_route)
    };

    let web = warp::serve(routes).run(([0, 0, 0, 0], 30080));

    let (discovery_sender, discovery_receiver) = mpsc::channel();

    let dht22_monitor = {
        let db = db.clone();

        tokio::spawn(async move {
            loop {
                match dht22::read(PIN) {
                    Ok(dht22::Reading {
                        temperature,
                        humidity,
                    }) => {
                        let now = Utc::now();
                        println!("at={:?} celsius={:?} humidity={:?}", now, temperature, humidity);
                        db.insert_dht22_reading(
                            "garage".into(),
                            (temperature * 100.0) as u16,
                            (humidity * 100.0) as u16,
                        );
                    }
                    Err(err) => {
                        let now = Utc::now();
                        eprintln!("at={:?} error={:?}", now, err);
                    }
                }

                tokio::time::delay_for(std::time::Duration::from_secs(15)).await
            }
        })
    };

    let disco = tokio::spawn(async move { BluetoothDiscovery::run(discovery_sender).await.unwrap() });

    let disco_processor = {
        let db = db.clone();

        tokio::spawn(async move {
            loop {
                match discovery_receiver.recv().unwrap() {
                    BluetoothDiscoveryEvent::DiscoveredTilt(tilt) => {
                        let now = Utc::now();
                        let centi_celsius = ((i32::from(tilt.fahrenheit) - 32) * 500) / 9;

                        println!(
                            "at={:?} which={:?} celsius={:?} gravity={:?}",
                            now, tilt.color, centi_celsius, tilt.gravity
                        );

                        db.insert_tilt_reading(&tilt);

                        tilts.write().unwrap().insert(tilt.color, DeviceInfo::new(now, tilt));
                    }

                    BluetoothDiscoveryEvent::DiscoveredGrainfather(gf_client) => {
                        gf.set_client(gf_client);
                    }
                }
            }
        })
    };

    web.await;
    disco.await.unwrap();
    disco_processor.await.unwrap();
    dht22_monitor.await.unwrap();
}

pub struct DeviceInfo<T> {
    when: DateTime<Utc>,
    device: T,
}

impl<T> DeviceInfo<T> {
    pub fn new(when: DateTime<Utc>, device: T) -> Self {
        Self {
            when,
            device,
        }
    }
}
