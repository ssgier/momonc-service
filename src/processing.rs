use std::sync::{Arc, Mutex};
use log::debug;

use crate::app_state::Spec;
use crate::app_state::AppState;

pub async fn process(spec: Spec, _obj_func_cmd: String, _app_state: Arc<Mutex<AppState>>) {
    debug!("Start processing spec: {:?}", spec);
}
