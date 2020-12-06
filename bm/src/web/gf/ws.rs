use bm_grainfather::btleplug::Client as GrainfatherClient;
use futures::{SinkExt, StreamExt};
use std::sync::{mpsc, Arc, RwLock};
use warp::ws::WebSocket;

pub struct GrainfatherWebSocketHandler {}

impl GrainfatherWebSocketHandler {
    pub async fn run(gf: Arc<RwLock<Option<GrainfatherClient>>>, ws: WebSocket) {
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
