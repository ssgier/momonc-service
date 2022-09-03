use crate::app_config;
use crate::app_state::AppState;
use crate::msg_handling::MsgHandler;
use env_logger;
use futures::StreamExt;
use futures::TryStreamExt;
use futures_util::future;
use log::{info, LevelFilter};
use serde_json;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;
use tokio_tungstenite::tungstenite::Message;

pub fn run() {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(app_config::NUM_WORKER_THREADS)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(start_server());
}

async fn start_server() {
    let app_state = Arc::new(Mutex::new(AppState::new()));
    let try_socket = TcpListener::bind(&app_config::ADDR).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on {}", app_config::ADDR);
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(
            stream,
            addr,
            MsgHandler::new(app_state.clone()),
        ));
    }
}

async fn handle_connection(tpc_stream: TcpStream, addr: SocketAddr, msg_handler: MsgHandler) {
    info!("Incoming TCP connection from: {}", addr);
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(tpc_stream).await {
        let (_outgoing, incoming) = ws_stream.split();

        incoming
            .try_for_each(|tungstenite_msg| {
                info!("Received msg: {}", tungstenite_msg);
                if let Message::Text(msg_str) = tungstenite_msg {
                    match serde_json::from_str(&msg_str) {
                        Ok(msg) => {
                            info!("Deserialized msg: {:?}", msg);
                            msg_handler.handle(msg);
                        },
                        Err(err) => {
                            info!("Unable to deserialize json: {}", err);
                        }
                    }
                } else {
                    info!("Invalid msg received: {:?}", tungstenite_msg);
                }
                future::ok(())
            })
            .await
            .unwrap_or_default();

        info!("{} disconnected", &addr);
    } else {
        info!("Websocket handshake failed");
    }
}
