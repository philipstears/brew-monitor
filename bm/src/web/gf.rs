mod ws;
use ws::GrainfatherWebSocketHandler;

use bm_grainfather::{self as gf, btleplug::Client as GrainfatherClient};
use std::sync::{Arc, RwLock};
use warp::{reject::Rejection, reply::Reply, ws::Ws, Filter};

pub fn route(
    gf: Arc<RwLock<Option<GrainfatherClient>>>,
) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
    let ws = {
        let gf = gf.clone();

        warp::path!("ws").and(warp::ws()).map(move |ws: Ws| {
            let gf = gf.clone();

            ws.on_upgrade(move |websocket| GrainfatherWebSocketHandler::run(gf, websocket))
        })
    };

    let command = {
        let gf = gf.clone();

        warp::path!("command").and(warp::post()).and(warp::body::json()).and_then(move |command: gf::Command| {
            let gf = gf.clone();

            async move {
                let guard = gf.read().unwrap();

                if let Some(client) = &*guard {
                    client.command(&command).unwrap();
                    Ok(warp::reply::json(&GrainfatherResponse {}))
                } else {
                    Err(warp::reject::not_found())
                }
            }
        })
    };

    let recipe = {
        let gf = gf.clone();

        warp::path!("recipe").and(warp::post()).and(warp::body::json()).and_then(move |recipe: gf::Recipe| {
            let gf = gf.clone();

            async move {
                let guard = gf.read().unwrap();

                if let Some(client) = &*guard {
                    client.send_recipe(&recipe).unwrap();
                    Ok(warp::reply::json(&GrainfatherResponse {}))
                } else {
                    Err(warp::reject::not_found())
                }
            }
        })
    };

    warp::path("gf").and(command.or(recipe).or(ws))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GrainfatherRequest {}

#[derive(serde::Serialize, serde::Deserialize)]
struct GrainfatherResponse {}
