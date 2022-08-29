use env_logger;
use log::{info, LevelFilter};
use serde_json::Value;
use tokio::runtime;
use std::env;

const NUM_WORKER_THREADS: usize = 2;

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

    let rt = runtime::Builder::new_multi_thread()
        .worker_threads(NUM_WORKER_THREADS)
        .build()
        .unwrap();

    rt.block_on(async {
        // TODO
    });
}
