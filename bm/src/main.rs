mod bluetooth_discovery;
pub use bluetooth_discovery::*;

mod grainfather_client;
pub use grainfather_client::*;

use bm_grainfather::*;
use bm_tilt::*;

use std::{collections::HashMap, sync::{mpsc, Arc, RwLock}};

use warp::{Filter};
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

#[derive(serde::Serialize, serde::Deserialize)]
struct TiltStatus {
    centi_celsius: i32,
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GrainfatherRequest {
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GrainfatherResponse {
}

#[tokio::main]
pub async fn main() {
    let tilts = Arc::new(RwLock::new(HashMap::<TiltColor, DeviceInfo<Tilt>>::new()));

    let gf: Arc<RwLock<Option<GrainfatherClient>>> = Arc::new(RwLock::new(None));

    let routes = {
        let hello = warp::path!("hello" / String)
            .map(|name| format!("Hello, {}!", name))
            .with(warp::reply::with::header("Content-Type", "application/json"));

        let gf_route = {
            let command = {
                let gf = gf.clone();

                warp::path!("command")
                    .and(warp::post())
                    .and(warp::body::json())
                    .and_then(move |command: GrainfatherCommand| {
                        let gf = gf.clone();

                        async move {
                            let guard = gf.read().unwrap();

                            if let Some(client) = &*guard {
                                client.command(&command).unwrap();
                                Ok(warp::reply::json(&GrainfatherResponse {}))
                            }
                            else {
                                Err(warp::reject::not_found())
                            }
                        }
                    })
            };

            let recipe = {
                let gf = gf.clone();

                warp::path!("recipe")
                    .and(warp::post())
                    .and(warp::body::json())
                    .and_then(move |recipe: Recipe| {
                        let gf = gf.clone();

                        async move {
                            let guard = gf.read().unwrap();

                            if let Some(client) = &*guard {
                                client.send_recipe(&recipe).unwrap();
                                Ok(warp::reply::json(&GrainfatherResponse {}))
                            }
                            else {
                                Err(warp::reject::not_found())
                            }
                        }
                    })
            };

            warp::path("gf").and(command.or(recipe))
        };

        let tilt_route = {
            let tilts = tilts.clone();

            warp::path!("tilt" / TiltColorParam)
                .and_then(move |color: TiltColorParam| {
                    let tilts = tilts.clone();

                    async move {
                        if let Some(info) = tilts.read().unwrap().get(color.color()) {
                            Ok(warp::reply::json(&TiltStatus {
                                centi_celsius: ((i32::from(info.device.fahrenheit) - 32) * 500) / 9,
                            }))
                        }
                        else {
                            Err(warp::reject::not_found())
                        }
                    }
                })
        };

        hello
            .or(tilt_route)
            .or(gf_route)
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

                BluetoothDiscoveryEvent::DiscoveredGrainfather(gf_client) => {
                    // gf_client.subscribe(Box::new(|notification| {
                    //     let now = Utc::now();
                    //     println!(
                    //         "at={:?} which=grainfather notification={:?}",
                    //         now, notification
                    //         );
                    // })).unwrap();

                    gf.write().unwrap().replace(gf_client);
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
