use bm_db::{DHT22Info, DB};
use chrono::{DateTime, Utc};
use futures::future;
use serde::{Deserialize, Serialize};
use warp::{reject::Rejection, reply::Reply, Filter};

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
            let maybe_dht22 = db.dht22().try_get_info(alias.as_str()).unwrap();

            maybe_dht22
                .map(|dht22| future::ok(warp::reply::json(&dht22)))
                .unwrap_or_else(|| future::err(warp::reject::not_found()))
        })
    };

    let put = {
        let db = db.clone();
        sensor
            .and(warp::path::end())
            .and(warp::filters::method::put())
            .and(warp::body::content_length_limit(1024))
            .and(warp::body::json())
            .map(move |_alias, body: DHT22Info| {
                db.dht22().upsert(&body).unwrap();
                // TODO: 204 if it already existed?
                warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::CREATED)
            })
    };

    let readings = sensor.and(warp::path!("readings")).and(warp::query::<ReadingsQuery>()).map(
        move |alias: String, query: ReadingsQuery| {
            // TODO: if there are no readings, we should check if the device is registered
            let readings = db.dht22().get_readings(alias.as_str(), query.from, query.to).unwrap();
            Ok(warp::reply::json(&readings))
        },
    );

    warp::path::path("dht22").and(get.or(put).or(readings))
}
