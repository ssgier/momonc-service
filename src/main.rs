use env_logger::{self, Builder};
use log::{debug, info, LevelFilter};
use momonc_service::obj_func::ObjFunc;
use serde::Serialize;
use std::env;

#[derive(Serialize)]
struct TestInput {
    x: f64,
    y: f64,
}

fn main() {
    Builder::from_default_env()
        .filter(None, LevelFilter::Debug)
        .init();
    let run_cmd =
        env::var("MOMONC_OBJ_FUNC").expect("Environment variable MOMONC_OBJ_FUNC not set");
    let obj_func = ObjFunc::new(&run_cmd);
    info!("Running with objective function run command: {}", &run_cmd);
    let test_val = TestInput { x: 0.5, y: 0.5 };
    let obj_val = obj_func.call(&test_val);
    debug!("Objective function value: {:?}", obj_val);
}
