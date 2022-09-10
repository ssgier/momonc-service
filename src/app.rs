use std::net::SocketAddr;

use env_logger;
use futures::SinkExt;
use futures::StreamExt;
use futures::TryStreamExt;
use futures_util::future;
use log::{info, LevelFilter};
use serde_json;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_tungstenite::tungstenite::Message;

use crate::domain::StatusMessage;
use crate::app_config;
use crate::app_state::run_app_fsm;
use crate::app_state::AppEvent;
use crate::disk_cache;
use crate::msg_handling::MsgHandler;
use crate::type_aliases::EventSender;

pub fn run() {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(app_config::NUM_WORKER_THREADS)
        .enable_all()
        .build()
        .unwrap();

    let (event_sender, event_receiver) = mpsc::unbounded_channel::<AppEvent>();
    let fsm = run_app_fsm(
        event_receiver,
        event_sender.clone(),
        disk_cache::retrieve_default_processing_job_data(),
    );

    let server = start_server(event_sender);

    rt.block_on(future::join(fsm, server));
}

async fn start_server(event_sender: EventSender) {
    let try_socket = TcpListener::bind(&app_config::ADDR).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on {}", app_config::ADDR);
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(stream, addr, event_sender.clone()));
    }
}

async fn handle_connection(tpc_stream: TcpStream, addr: SocketAddr, event_sender: EventSender) {
    info!("Incoming TCP connection from: {}", addr);
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(tpc_stream).await {
        let (mut outgoing, incoming) = ws_stream.split();

        let in_msg_handler = MsgHandler::new(event_sender.clone());

        let (subscription, subscriber) = mpsc::unbounded_channel::<StatusMessage>();
        event_sender
            .send(AppEvent::NewSubscriber(subscription.clone()))
            .unwrap();

        let mut out_msgs = UnboundedReceiverStream::new(subscriber)
            .map(|status_msg| Ok(Message::Text(serde_json::to_string(&status_msg).unwrap())));

        let out_fut = outgoing.send_all(&mut out_msgs);

        let in_fut = incoming.try_for_each(|tungstenite_msg| {
            info!("Received msg: {}", tungstenite_msg);
            if let Message::Text(msg_str) = tungstenite_msg {
                match serde_json::from_str(&msg_str) {
                    Ok(msg) => {
                        info!("Deserialized msg: {:?}", msg);
                        in_msg_handler.handle(msg);
                    }
                    Err(err) => {
                        info!("Unable to deserialize json: {}", err);
                    }
                }
            }
            future::ok(())
        });

        match future::join(out_fut, in_fut).await {
            (o, i) => {
                o.unwrap();
                i.unwrap();
            }
        }
        info!("{} disconnected", &addr);
    } else {
        info!("Websocket handshake failed");
    }
}
