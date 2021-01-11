use crate::DeviceInfo;
use bm_db::DB;
use bm_tilt::{Tilt, TiltColor};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};
use warp::{reject::Rejection, reply::Reply, Filter};

#[derive(Deserialize, Serialize)]
struct ReadingsQuery {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

pub fn route(
    db: DB,
    tilts: Arc<RwLock<HashMap<TiltColor, DeviceInfo<Tilt>>>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let readings = warp::path!("tilt" / TiltColorParam).and(warp::query::<ReadingsQuery>()).map(
        move |color: TiltColorParam, query: ReadingsQuery| {
            let readings = db.tilt_ensure(color.color()).get_readings(query.from, query.to).unwrap();
            Ok(warp::reply::json(&readings))
        },
    );

    let single = warp::path!("tilt" / TiltColorParam).and_then(move |color: TiltColorParam| {
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
    });

    readings.or(single)
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
