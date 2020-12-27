mod bluetooth_discovery;
pub use bluetooth_discovery::*;

mod devices;
use devices::gf_manager::GrainfatherManager;

mod web;

use bm_db::DB;
use bm_tilt::*;
use chrono::prelude::*;
use dht22_pi as dht22;
use log::error;
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
        let garage = db.get_dht22("garage".into());

        tokio::spawn(async move {
            loop {
                match dht22::read(PIN) {
                    Ok(dht22::Reading {
                        temperature,
                        humidity,
                    }) => {
                        let now = Utc::now();
                        println!("at={:?} celsius={:?} humidity={:?}", now, temperature, humidity);

                        if let Err(err) = garage.insert_reading((temperature * 100.0) as u16, (humidity * 100.0) as u16)
                        {
                            error!(
                                "Unable to insert dht22 reading for {} with temperature {} and humidity {}: {:?}",
                                "garage", temperature, humidity, err,
                            );
                        }
                    }

                    Err(dht22::ReadingError::Gpio(rppal::gpio::Error::UnknownModel)) => {
                        error!("Unable to read DHT22 on pin {}, we can't determine the model of raspberry pi, perhaps this isn't one?", PIN);
                        return;
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

    let disco = tokio::spawn(async move {
        match BluetoothDiscovery::run(discovery_sender).await {
            Ok(()) => {}

            Err(bluez::Error::CommandError {
                opcode: bluez::interface::Command::StartServiceDiscovery,
                status: bluez::interface::CommandStatus::PermissionDenied,
            }) => {
                error!("Unable to start bluetooth discovery because permission was denied, make sure the permissions are properly enabled.");
            }

            Err(other) => {
                error!("Unable to start bluetooth discovery for an unknown reason: {:?}", other);
            }
        }
    });

    let disco_processor = {
        let db = db.clone();

        tokio::spawn(async move {
            loop {
                let event = match discovery_receiver.recv() {
                    Ok(event) => event,

                    Err(std::sync::mpsc::RecvError) => {
                        // The discovery process has gone down, this will be logged elsewhere
                        return;
                    }
                };

                match event {
                    BluetoothDiscoveryEvent::DiscoveredTilt(tilt) => {
                        let now = Utc::now();
                        let centi_celsius = ((i32::from(tilt.fahrenheit) - 32) * 500) / 9;

                        println!(
                            "at={:?} which={:?} celsius={:?} gravity={:?}",
                            now, tilt.color, centi_celsius, tilt.gravity
                        );

                        // TODO: cache tilts
                        if let Err(err) = db.get_tilt(&tilt.color).insert_reading(tilt.fahrenheit, tilt.gravity) {
                            error!("Unable to insert tilt reading {:?}: {:?}", tilt, err);
                        }

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
