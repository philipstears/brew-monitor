mod ws;
use ws::GrainfatherWebSocketHandler;

use crate::devices::gf_manager::GrainfatherManager;
use crate::web::warp_helpers;

use bm_db::DB;
use bm_grainfather::{self as gf};
use serde::{Deserialize, Serialize};
use warp::{
    filters::BoxedFilter,
    reject::Rejection,
    reply::{Reply, Response},
    ws::Ws,
    Filter,
};

#[derive(Debug, Serialize, Deserialize)]
struct ActivateRecipeBody {
    name: String,
}

pub fn route(db: DB, gf: GrainfatherManager) -> BoxedFilter<(impl Reply,)> {
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
        let db = db.clone();
        let gf = gf.clone();

        warp::path!("recipe")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp_helpers::with(gf))
            .and(warp_helpers::with(db))
            .and_then(handlers::recipe_activate)
    };

    warp::path("gf").and(command.or(recipe).or(ws)).boxed()
}

mod handlers {
    use super::*;

    pub(super) async fn recipe_activate(
        body: ActivateRecipeBody,
        gf: GrainfatherManager,
        db: DB,
    ) -> Result<Response, Rejection> {
        let reply = match db.recipe().get_recipe_latest(&body.name) {
            Ok(Some(recipe)) => match gf.send_recipe(&recipe) {
                Ok(()) => warp::reply::json(&GrainfatherResponse {}).into_response(),
                Err(btleplug::Error::NotConnected) => {
                    warp::reply::with_status(warp::reply(), warp::http::StatusCode::BAD_REQUEST).into_response()
                }
                Err(error) => {
                    eprintln!("Unhandled bluetooth error {:?}", error);
                    warp::reply::with_status(warp::reply(), warp::http::StatusCode::INTERNAL_SERVER_ERROR)
                        .into_response()
                }
            },

            Ok(None) => warp::reply::with_status(warp::reply(), warp::http::StatusCode::NOT_FOUND).into_response(),

            Err(error) => {
                eprintln!("Unhandled data access error {:?}", error);
                warp::reply::with_status(warp::reply(), warp::http::StatusCode::INTERNAL_SERVER_ERROR).into_response()
            }
        };

        Ok(reply)
    }
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
