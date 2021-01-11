use bm_db::DB;
use chrono::{DateTime, Utc};
use futures::future;
use serde::{Deserialize, Serialize};
use warp::{reject::Rejection, reply::Reply, Filter};

#[derive(Deserialize, Serialize)]
struct Sensor {
    alias: String,
    pin: u32,
}

#[derive(Deserialize, Serialize)]
struct ReadingsQuery {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

pub fn route(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let sensor = warp::path::param::<String>();

    let get = {
        let db = db.clone();

        sensor.and(warp::path::end()).and(warp::filters::method::get()).and_then(move |alias: String| {
            let maybe_dht22 = db.dht22_try_get(alias.as_str()).unwrap();

            maybe_dht22
                .map(|dht22| {
                    let sensor = Sensor {
                        alias,
                        pin: dht22.get_pin().unwrap(),
                    };

                    future::ok(warp::reply::json(&sensor))
                })
                .unwrap_or_else(|| future::err(warp::reject::not_found()))
        })
    };

    let put = sensor
        .and(warp::path::end())
        .and(warp::filters::method::put())
        .and(warp::body::content_length_limit(1024))
        .and(warp::body::json())
        .map(|_alias, _body: std::collections::HashMap<String, String>| "Yeah boi!");

    let readings = sensor.and(warp::path!("readings")).and(warp::query::<ReadingsQuery>()).and_then(
        move |alias: String, query: ReadingsQuery| {
            let maybe_dht22 = db.dht22_try_get(alias.as_str()).unwrap();

            maybe_dht22
                .map(|dht22| {
                    let readings = dht22.get_readings(query.from, query.to).unwrap();
                    future::ok(warp::reply::json(&readings))
                })
                .unwrap_or_else(|| future::err(warp::reject::not_found()))
        },
    );

    warp::path::path("dht22").and(get.or(put).or(readings))
}
