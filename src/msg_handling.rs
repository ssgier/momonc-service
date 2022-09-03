use crate::api::Message::{self, *};
use crate::api::ProcessingJobData;
use crate::app_state::AppEvent;
use crate::app_state::AppState;
use crate::obj_func::ObjFuncCallDef;
use crate::param::ParamsSpec;
use log::info;
use std::fs;
use std::sync::Arc;
use std::sync::Mutex;

pub struct MsgHandler {
    app_state: Arc<Mutex<AppState>>,
}

impl MsgHandler {
    pub fn new(app_state: Arc<Mutex<AppState>>) -> MsgHandler {
        MsgHandler { app_state }
    }

    pub fn handle(&self, msg: Message) {
        match msg {
            ProcessingJobDataMsg(processing_job_data) => {
                self.handle_processing_job(processing_job_data)
            }
            StopProcessingMsg => {
                tokio::spawn(AppState::on_event(
                    self.app_state.clone(),
                    AppEvent::RequestStop,
                ));
            }
        }
    }

    fn handle_processing_job(&self, processing_job_data: ProcessingJobData) {
        let spec_json_str =
            fs::read_to_string(processing_job_data.spec_file).expect("Unable to read spec file");
        let spec_json: serde_json::Value =
            serde_json::from_str(&spec_json_str).expect("Unable to deserialize json");

        let spec = ParamsSpec::from_json(spec_json).unwrap();

        let obj_func_call_def = ObjFuncCallDef {
            program: processing_job_data.program,
            args: processing_job_data.args,
        };

        info!(
            "Objective function call definition: {:?}",
            obj_func_call_def
        );

        tokio::spawn(AppState::on_event(
            self.app_state.clone(),
            AppEvent::ProcessingJob(spec, processing_job_data.algo_conf, obj_func_call_def),
        ));
    }
}
