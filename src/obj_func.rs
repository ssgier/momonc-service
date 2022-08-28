use serde::{Deserialize, Serialize};
use serde_json;
use std::process::Command;
use std::process::Stdio;

pub struct ObjFunc {
    run_cmd: String,
}

#[derive(Serialize)]
struct TmpOb {
    x: f64,
}
#[derive(Debug, Deserialize)]
struct ObjFuncChildResult {
    obj_func_val: f64,
}

#[derive(Debug)]
pub struct ObjFuncEvalError {
    msg: String,
}

impl ObjFunc {
    // TODO: error handling
    // TODO: async
    pub fn new(run_cmd: &str) -> ObjFunc {
        ObjFunc {
            run_cmd: run_cmd.to_string(),
        }
    }

    pub fn call<T: Serialize>(&self, params: &T) -> Result<f64, ObjFuncEvalError> {
        let child = Command::new(&self.run_cmd)
            .arg(serde_json::to_string(params).unwrap())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect(format!("Failed to execute run cmd {}", &self.run_cmd).as_str());
        let pid = child.id();
        let output = child.wait_with_output().unwrap();
        if !output.stderr.is_empty() {
            let error_msg = String::from_utf8(output.stderr);
            let error = ObjFuncEvalError {
                msg: error_msg.unwrap_or(format!("Objective function process (PID: {}) wrote data to stderr (was not valid utf8)", pid))
            };

            Err(error)
        } else {
            let result: ObjFuncChildResult = serde_json::from_slice(&output.stdout).unwrap();
            Ok(result.obj_func_val)
        }
    }
}
