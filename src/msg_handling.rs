use crate::app_state::AppEvent;
use crate::domain::ProcessingJobData;
use crate::domain::RequestMessage::{self, *};
use crate::obj_func::ObjFuncCallDef;
use crate::param::ParamsSpec;
use crate::type_aliases::EventSender;
use log::info;
use std::fs;

#[derive(Debug)]
pub struct MsgHandler {
    event_sender: EventSender,
}

impl MsgHandler {
    pub fn new(event_sender: EventSender) -> MsgHandler {
        MsgHandler { event_sender }
    }

    pub fn handle(&self, msg: RequestMessage) {
        match msg {
            ProcessingJobDataMsg(processing_job_data) => {
                self.handle_processing_job(processing_job_data)
            }
            StopProcessingMsg => {
                self.event_sender.send(AppEvent::RequestStop).unwrap();
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

        self.event_sender
            .send(AppEvent::ProcessingJob(
                spec,
                processing_job_data.algo_conf,
                obj_func_call_def,
            ))
            .unwrap();
    }
}
