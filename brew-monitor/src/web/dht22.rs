use bm_db::{DHT22Info, DB};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use warp::{reject::Rejection, reply::Reply, Filter};

#[derive(Deserialize, Serialize)]
struct ReadingsQuery {
    from: DateTime<Utc>,
    to: DateTime<Utc>,
}

pub fn route(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    warp::path::path("dht22").and(routes::put(db.clone()).or(routes::get(db.clone())).or(routes::readings(db.clone())))
}

fn with_db(db: DB) -> impl Filter<Extract = (DB,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

mod routes {
    use super::*;

    pub(super) fn get(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(String).and(warp::filters::method::get()).and(with_db(db)).and_then(handlers::get)
    }

    pub(super) fn put(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(String)
            .and(warp::filters::method::put())
            .and(warp::body::content_length_limit(1024))
            .and(warp::body::json())
            .map(move |_alias, body: DHT22Info| {
                db.dht22().upsert(&body).unwrap();
                // TODO: 204 if it already existed?
                warp::reply::with_status(warp::reply::reply(), warp::http::StatusCode::CREATED)
            })
    }

    pub(super) fn readings(db: DB) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        warp::path!(String / "readings").and(warp::query::<ReadingsQuery>()).map(
            move |alias: String, query: ReadingsQuery| {
                // TODO: if there are no readings, we should check if the device is registered
                let readings = db.dht22().get_readings(alias.as_str(), query.from, query.to).unwrap();
                Ok(warp::reply::json(&readings))
            },
        )
    }
}

mod handlers {
    use super::*;

    pub(super) async fn get(alias: String, db: DB) -> Result<Box<dyn Reply>, Rejection> {
        let reply: Box<dyn Reply> = match db.dht22().try_get_info(alias.as_str()).unwrap() {
            Some(dht22) => Box::new(warp::reply::json(&dht22)),
            None => Box::new(warp::reply::with_status(warp::reply(), warp::http::StatusCode::NOT_FOUND)),
        };

        Ok(reply)
    }
}
