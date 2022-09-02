use crate::algo::AlgoConf::ParallelHillClimbing;
use crate::algo::ParallelHillClimbingConf;
use crate::api::Message as ApiMessage;
use crate::api::Message::*;
use crate::app_config;
use crate::app_state::AppEvent;
use crate::app_state::AppState;
use crate::obj_func::ObjFuncCallDef;
use env_logger;
use futures::StreamExt;
use futures::TryStreamExt;
use futures_util::future;
use log::{info, LevelFilter};
use serde_json;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;
use tokio_tungstenite::tungstenite::Message;
use crate::param::ParamsSpec;

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
        tokio::spawn(handle_connection(app_state.clone(), stream, addr));
    }
}

async fn handle_connection(
    app_state: Arc<Mutex<AppState>>,
    tpc_stream: TcpStream,
    addr: SocketAddr,
) {
    info!("Incoming TCP connection from: {}", addr);
    if let Ok(ws_stream) = tokio_tungstenite::accept_async(tpc_stream).await {
        let (_outgoing, incoming) = ws_stream.split();

        // TODO: clean up
        // Address connection handler panics (prevent them and/or transition to terminal/error
        // state if they happen)
        incoming
            .try_for_each(|tungstenite_msg| {
                info!("Received msg: {}", tungstenite_msg);
                if let Message::Text(msg_str) = tungstenite_msg {
                    let msg: ApiMessage = serde_json::from_str(&msg_str).unwrap();
                    info!("Deserialized msg: {:?}", msg);

                    match msg {
                        ProcessingJobDataMsg(processing_job_data) => {
                            let spec_json_str = fs::read_to_string(processing_job_data.spec_file)
                                .expect("Unable to read spec file");
                            let spec_json: serde_json::Value = serde_json::from_str(&spec_json_str)
                                .expect("Unable to deserialize json");

                            let spec = ParamsSpec::from_json(spec_json).unwrap();

                            let obj_func_call_def = ObjFuncCallDef {
                                program: processing_job_data.program,
                                args: processing_job_data.args,
                            };

                            info!(
                                "Objective function call definition: {:?}",
                                obj_func_call_def
                            );

                            let algo_conf = ParallelHillClimbing(ParallelHillClimbingConf {
                                std_dev: 0.01,
                                num_values_per_iter: 10,
                            });
                            tokio::spawn(AppState::on_event(
                                app_state.clone(),
                                AppEvent::ProcessingJob(spec, algo_conf, obj_func_call_def),
                            ));
                        }
                    }
                }
                future::ok(())
            })
            .await
            .unwrap();

        info!("{} disconnected", &addr);

        let app_state_clone = app_state.clone();
        AppState::on_event(app_state_clone, AppEvent::RequestStop())
            .await
            .unwrap();
    } else {
        info!("Websocket handshake failed");
    }
}
