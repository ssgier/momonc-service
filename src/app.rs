use crate::algo::AlgoConf::ParallelHillClimbing;
use crate::algo::ParallelHillClimbingConf;
use crate::app_state::AppEvent;
use crate::app_state::AppState;
use crate::app_config;
use env_logger;
use log::{info, LevelFilter};
use serde_json;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::Duration;
use tokio::net::{TcpListener, TcpStream};
use tokio::runtime;
use crate::obj_func::ObjFuncCallDef;
use std::path::Path;

use crate::param::{Dim, DimSpecWithBounds, ParamsSpec};

pub fn run() {
    env_logger::Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();

    let momonc_dir_str = env::var("MOMONC_DIR").expect("Environment variable MOMONC_DIR not set");
    let momonc_dir = Path::new(&momonc_dir_str);
    let spec_json_str = fs::read_to_string(momonc_dir.join("spec.json")).expect("Unable to read spec file");
    let spec_json: serde_json::Value = serde_json::from_str(&spec_json_str).expect("Unable to deserialize json");
    let obj_func_script = momonc_dir.join("obj_func_mock.py");
    let obj_func_call_def = ObjFuncCallDef {
        program: "python".to_string(),
        args: vec![obj_func_script.to_str().unwrap().to_string()],
    };

    info!("Objective function call definition: {:?}", obj_func_call_def);
    info!("spec_json: {:?}", spec_json);

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

    rt.block_on(start_server(spec, obj_func_call_def));
}

async fn start_server(default_spec: ParamsSpec, default_obj_func_cmd: ObjFuncCallDef) {
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
