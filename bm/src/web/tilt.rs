use crate::DeviceInfo;
use bm_tilt::{Tilt, TiltColor};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use warp::{reject::Rejection, reply::Reply, Filter};

pub fn route(
    tilts: Arc<RwLock<HashMap<TiltColor, DeviceInfo<Tilt>>>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let tilts = tilts.clone();

    warp::path!("tilt" / TiltColorParam).and_then(move |color: TiltColorParam| {
        let tilts = tilts.clone();

        async move {
            if let Some(info) = tilts.read().unwrap().get(color.color()) {
                Ok(warp::reply::json(&TiltStatus {
                    centi_celsius: ((i32::from(info.device.fahrenheit) - 32) * 500) / 9,
                }))
            } else {
                Err(warp::reject::not_found())
            }
        }
    })
}

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
