use crate::devices::gf_manager::GrainfatherManager;
use futures::{SinkExt, StreamExt};
use warp::ws::WebSocket;

pub struct GrainfatherWebSocketHandler {}

impl GrainfatherWebSocketHandler {
    pub async fn run(mut gf: GrainfatherManager, ws: WebSocket) {
        let (mut ws_tx, _ws_rx) = ws.split();

        let gf_rx = gf.subscribe();

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
