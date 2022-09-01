use crate::algo::AlgoConf::ParallelHillClimbing;
use crate::algo::ParallelHillClimbingConf;
use crate::app_state::AppEvent;
use crate::app_state::AppState;
use crate::app_config;
use env_logger;
use log::{info, LevelFilter};
use serde_json::Value;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;

use crate::param::{Dim, DimSpecWithBounds, ParamsSpec};

pub fn run() {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();

    let run_cmd =
        env::var("MOMONC_OBJ_FUNC").expect("Environment variable MOMONC_OBJ_FUNC not set");

    let spec_str = env::var("MOMONC_SPEC").expect("Environment variable MOMONC_SPEC not set");
    let spec = serde_json::from_str::<Value>(&spec_str).expect("Failed to deserialize spec json");

    info!(
        "Application starting.\nObjective function command:\n\n{}\n\nSpec:\n\n{}",
        run_cmd,
        serde_json::to_string_pretty(&spec).unwrap()
    );

    let spec = ParamsSpec {
        dims: vec![
            Dim::RealNumber(DimSpecWithBounds::new("x".to_string(), 0.1, 0.0, 1.0)),
            Dim::RealNumber(DimSpecWithBounds::new("y".to_string(), 0.2, 0.0, 1.0)),
        ],
    }; // TODO remove, just a test

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(app_config::NUM_WORKER_THREADS)
        .enable_all()
        .build()
        .unwrap();

    rt.block_on(start_server(spec, run_cmd));
}

async fn start_server(default_spec: ParamsSpec, default_obj_func_cmd: String) {
    let app_state = Arc::new(Mutex::new(AppState::new()));

    // TODO: remove, just a test
    let algo_conf = ParallelHillClimbing(ParallelHillClimbingConf {
        std_dev: 0.01,
        num_values_per_iter: 10,
    });
    AppState::on_event(
        app_state.clone(),
        AppEvent::ProcessingJob(default_spec, algo_conf, default_obj_func_cmd),
    )
    .await
    .unwrap();

    let app_state_clone = app_state.clone();
    tokio::time::sleep(Duration::from_millis(5000000)).await;
    AppState::on_event(app_state_clone, AppEvent::RequestStop())
        .await
        .unwrap();
    // end of test

    let try_socket = TcpListener::bind(&app_config::ADDR).await;
    let listener = try_socket.expect("Failed to bind");
    info!("Listening on {}", app_config::ADDR);
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(app_state.clone(), stream, addr));
    }
}

async fn handle_connection(
    _app_state: Arc<Mutex<AppState>>,
    _tpc_stream: TcpStream,
    _addr: SocketAddr,
) {
    panic!("Not implemented");
}
