use bm_db::DB;
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
    let readings = warp::path!("dht22" / String).and(warp::query::<ReadingsQuery>()).and_then(
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

    readings
}
