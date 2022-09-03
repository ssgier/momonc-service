use serde::{Deserialize, Serialize};
use crate::algo::*;

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    ProcessingJobDataMsg(ProcessingJobData),
    StopProcessingMsg
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessingJobData {
    pub program: String,
    pub args: Vec<String>,
    pub spec_file: String,
    pub algo_conf: AlgoConf
}
