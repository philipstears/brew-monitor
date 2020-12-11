mod bluetooth_discovery;
pub use bluetooth_discovery::*;

mod devices;
use devices::gf_manager::GrainfatherManager;

mod web;

use bm_tilt::*;
use chrono::prelude::*;
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, RwLock},
};
use warp::Filter;

#[tokio::main]
pub async fn main() {
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

    let disco = tokio::spawn(async move { BluetoothDiscovery::run(discovery_sender).await.unwrap() });

    let disco_processor = tokio::spawn(async move {
        loop {
            match discovery_receiver.recv().unwrap() {
                BluetoothDiscoveryEvent::DiscoveredTilt(tilt) => {
                    let now = Utc::now();
                    let centi_celsius = ((i32::from(tilt.fahrenheit) - 32) * 500) / 9;

                    println!(
                        "at={:?} which={:?} celsius={:?} gravity={:?}",
                        now, tilt.color, centi_celsius, tilt.gravity
                    );

                    tilts.write().unwrap().insert(tilt.color, DeviceInfo::new(now, tilt));
                }

                BluetoothDiscoveryEvent::DiscoveredGrainfather(gf_client) => {
                    gf.set_client(gf_client);
                }
            }
        }
    });

    web.await;
    disco.await.unwrap();
    disco_processor.await.unwrap();
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
