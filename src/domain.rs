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
    Time(f64),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DomainState {
    Idle(DefaultProcessingJobData),
    Processing(ProcessingState),
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
pub struct ProcessingState {
    pub recent_candidate_eval_reports: Vec<CandidateEvalReport>,
    pub best_seen_candidate_eval_reports: Vec<CandidateEvalReport>,
    pub time: f64,
    pub window_length_hint: usize,
    pub best_seen_table_size_hint: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CandidateEvalReport {
    pub start_time: f64,
    pub start_unix_timestamp: f64,
    pub completion_time: f64,
    pub obj_func_val: Option<f64>,
    pub best_seen_obj_func_val_before: Option<f64>,
    pub candidate: serde_json::Value,
    pub latest_interleaving_completion_time: Option<f64>
}
