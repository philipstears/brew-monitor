mod ws;
use ws::GrainfatherWebSocketHandler;

use crate::devices::gf_manager::GrainfatherManager;

use bm_grainfather::{self as gf};
use warp::{reject::Rejection, reply::Reply, ws::Ws, Filter};

pub fn route(gf: GrainfatherManager) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
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
                gf.command(&command)
                    .map(|()| warp::reply::json(&GrainfatherResponse {}))
                    .map_err(|error| btleplug_to_warp_error(error))
            }
        })
    };

    let recipe = {
        let gf = gf.clone();

        warp::path!("recipe").and(warp::post()).and(warp::body::json()).and_then(move |recipe: gf::Recipe| {
            let gf = gf.clone();

            async move {
                gf.send_recipe(&recipe)
                    .map(|()| warp::reply::json(&GrainfatherResponse {}))
                    .map_err(|error| btleplug_to_warp_error(error))
            }
        })
    };

    warp::path("gf").and(command.or(recipe).or(ws))
}

#[derive(serde::Serialize, serde::Deserialize)]
struct GrainfatherRequest {}

#[derive(serde::Serialize, serde::Deserialize)]
struct GrainfatherResponse {}

fn btleplug_to_warp_error(error: btleplug::Error) -> Rejection {
    match error {
        btleplug::Error::NotConnected => warp::reject::not_found(),
        _ => panic!("Unhandled bluetooth error {:?}", error),
    }
}
