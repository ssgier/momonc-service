use serde::{Deserialize, Serialize};
use crate::algo::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum RequestMessage {
    ProcessingJobDataMsg(ProcessingJobData),
    StopProcessingMsg
}

#[derive(Serialize, Deserialize, Debug)]
pub enum StatusMessage {
    DomainState(DomainState),
}

#[derive(Serialize, Deserialize, Debug)]
pub enum DomainState {
    Idle(DefaultProcessingJobData),
    Processing,
    Terminal,
    Error
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DefaultProcessingJobData(pub ProcessingJobData);

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProcessingJobData {
    pub program: String,
    pub args: Vec<String>,
    pub spec_file: String,
    pub algo_conf: AlgoConf
}
