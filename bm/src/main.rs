mod bluetooth_discovery;
pub use bluetooth_discovery::*;

mod grainfather_client;
pub use grainfather_client::*;

mod server;
pub use server::*;

use bm_grainfather::*;
use bm_tilt::*;

use std::{collections::HashMap, sync::{mpsc, Arc, RwLock}, thread, time::Duration};

use warp::{http, Filter, Rejection, Reply};
use async_std::task::block_on;
use chrono::prelude::*;

struct TiltColorParam(TiltColor);

impl TiltColorParam {
    pub fn color(&self) -> &TiltColor {
        &self.0
    }
}

impl std::convert::Into<TiltColor> for TiltColorParam {
    fn into(self) -> TiltColor {
        self.0
    }
}

struct InvalidTiltColor;

impl std::str::FromStr for TiltColorParam {
    type Err = InvalidTiltColor;

    fn from_str(other: &str) -> Result<Self, Self::Err> {
        match other {
            "red" => Ok(Self(TiltColor::Red)),
            "green" => Ok(Self(TiltColor::Green)),
            "black" => Ok(Self(TiltColor::Black)),
            "purple" => Ok(Self(TiltColor::Purple)),
            "orange" => Ok(Self(TiltColor::Orange)),
            "blue" => Ok(Self(TiltColor::Blue)),
            "yellow" => Ok(Self(TiltColor::Yellow)),
            "pink" => Ok(Self(TiltColor::Pink)),
            _ => Err(InvalidTiltColor),
        }
    }
}

#[tokio::main]
pub async fn main() {
    let tilts = Arc::new(RwLock::new(HashMap::<TiltColor, DeviceInfo<Tilt>>::new()));

    let routes = {
        let hello = warp::path!("hello" / String)
            .map(|name| format!("Hello, {}!", name))
            .with(warp::reply::with::header("Content-Type", "application/json"));

        let tilts = tilts.clone();

        let tilt = warp::path!("tilt" / TiltColorParam)
            .map(move |ref color: TiltColorParam| {
                if let Some(info) = tilts.read().unwrap().get(color.color()) {
                    format!("{}", info.device.fahrenheit)
                }
                else {
                    "".into()
                }
            })
            .with(warp::reply::with::header("Content-Type", "application/json"));

        hello.or(tilt)
    };

    let web = warp::serve(routes).run(([0, 0, 0, 0], 3030));

    let (discovery_sender, discovery_receiver) = mpsc::channel();

    let disco = tokio::spawn(async move {
        BluetoothDiscovery::run(discovery_sender).await.unwrap()
    });

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

                _ => {
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
