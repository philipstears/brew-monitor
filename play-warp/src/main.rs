use serde::{Deserialize, Serialize};
use std::sync::{Arc, RwLock};
use warp::{http, Filter, Rejection, Reply};

struct Grainfather {
    heat_enabled: bool,
}

impl Default for Grainfather {
    fn default() -> Self {
        Self {
            heat_enabled: false,
        }
    }
}

impl Grainfather {
    pub fn set_enabled(&mut self, enabled: bool) {
        self.heat_enabled = enabled;
    }

    pub fn get_enabled(&self) -> bool {
        self.heat_enabled
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Heat {
    enabled: bool,
}

async fn set_heat(heat: Heat, gf: Arc<RwLock<Grainfather>>) -> Result<impl warp::Reply, warp::Rejection> {
    gf.write().unwrap().set_enabled(heat.enabled);

    Ok(warp::reply::with_status("Added items to the grocery list", http::StatusCode::CREATED))
}

async fn get_heat(gf: Arc<RwLock<Grainfather>>) -> Result<impl warp::Reply, warp::Rejection> {
    let enabled = Heat {
        enabled: gf.read().unwrap().get_enabled(),
    };

    Ok(warp::reply::json(&enabled))
}

fn json_body() -> impl Filter<Extract = (Heat,), Error = warp::Rejection> + Clone {
    // When accepting a body, we want a JSON body
    // (and to reject huge payloads)...
    warp::body::content_length_limit(1024 * 16).and(warp::body::json())
}

#[tokio::main]
async fn main() {
    let gf = Arc::new(RwLock::new(Grainfather::default()));
    let gf_filter = warp::any().map(move || gf.clone());

    let set_heat = warp::post().and(warp::path!("gf")).and(json_body()).and(gf_filter.clone()).and_then(set_heat);

    let get_heat = warp::get().and(warp::path!("gf")).and(gf_filter.clone()).and_then(get_heat);

    // GET /hello/warp => 200 OK with body "Hello, warp!"
    let hello = warp::path!("hello" / String)
        .map(|name| format!("Hello, {}!", name))
        .with(warp::reply::with::header("Content-Type", "application/json"));

    let two = warp::path!("two").map(|| "two");

    let routes = hello.or(two).or(set_heat).or(get_heat);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
