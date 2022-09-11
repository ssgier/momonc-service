use crate::algo::*;
use serde::{Deserialize, Serialize};
use serde_json;

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestMessage {
    StartProcessing(ProcessingJobData),
    StopProcessing,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StatusMessage {
    DomainState(DomainState),
    CandidateEvalReport(CandidateEvalReport),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum DomainState {
    Idle(DefaultProcessingJobData),
    Processing,
    Terminal,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultProcessingJobData(pub ProcessingJobData);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessingJobData {
    pub program: String,
    pub args: Vec<String>,
    pub spec_file: String,
    pub algo_conf: AlgoConf,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CandidateEvalReport {
    pub start_time: f64,
    pub completion_time: f64,
    pub obj_func_val: Option<f64>,
    pub candidate: serde_json::Value
}

