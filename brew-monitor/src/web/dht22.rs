use bm_db::DB;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use warp::{reject::Rejection, reply::Reply, Filter};

#[derive(Deserialize, Serialize)]
struct ReadingsQuery {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

pub fn route(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let readings = warp::path!("dht22" / String).and(warp::query::<ReadingsQuery>()).map(
        move |alias: String, query: ReadingsQuery| {
            let maybe_dht22 = db.dht22_try_get(alias.as_str()).unwrap();
            let dht22 = maybe_dht22.unwrap();
            let readings = dht22.get_readings(query.from, query.to).unwrap();
            Ok(warp::reply::json(&readings))
        },
    );

    readings
}
