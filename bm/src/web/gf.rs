use bm_grainfather::{self as gf, btleplug::Client as GrainfatherClient};
use futures::{SinkExt, StreamExt};
use std::sync::{mpsc, Arc, RwLock};
use warp::{
    reject::Rejection,
    reply::Reply,
    ws::{WebSocket, Ws},
    Filter,
};

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

struct GrainfatherWebSocketHandler {}

impl GrainfatherWebSocketHandler {
    async fn run(gf: Arc<RwLock<Option<GrainfatherClient>>>, ws: WebSocket) {
        let (mut ws_tx, _ws_rx) = ws.split();
        let (gf_tx, gf_rx) = mpsc::channel();

        {
            let guard = gf.read().unwrap();

            if let Some(client) = &*guard {
                client
                    .subscribe(Box::new(move |notification| {
                        if let Err(e) = gf_tx.send(notification) {
                            //
                        }
                    }))
                    .unwrap();
            }
        }

        loop {
            let notification = gf_rx.recv().unwrap();
            let json = serde_json::to_string(&notification).unwrap();
            let message = warp::ws::Message::text(json);

            if let Err(e) = ws_tx.send(message).await {
                println!("Error occurred sending to socket {:?}", e);
                return;
            }
        }
    }
}
