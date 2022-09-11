use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Stdio;

use tokio::process::Command;

#[derive(Debug, Deserialize)]
struct ObjFuncChildResult {
    obj_func_val: f64,
}

#[derive(Debug)]
pub struct ObjFuncCallDef {
    pub program: String,
    pub args: Vec<String>,
}

pub async fn call<T: Serialize>(call_def: &ObjFuncCallDef, params: &T) -> Option<f64> {
    let child = Command::new(&call_def.program)
        .args(&call_def.args)
        .arg(serde_json::to_string(&params).unwrap())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect(format!("Failed to execute: {:?}", call_def).as_str());
    let output = child.wait_with_output().await.unwrap();
    if !output.stderr.is_empty() {
        // TODO: error handling
        None
    } else {
        let result: ObjFuncChildResult = serde_json::from_slice(&output.stdout).unwrap();
        Some(result.obj_func_val)
    }
}
