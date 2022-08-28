use env_logger::{self, Builder};
use log::{info, LevelFilter};
use serde_json::Value;
use std::env;

#[tokio::main]
async fn main() {
    Builder::from_default_env()
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
}
