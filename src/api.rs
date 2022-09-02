use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub enum Message {
    ProcessingJobDataMsg(ProcessingJobData)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ProcessingJobData {
    pub program: String,
    pub args: Vec<String>,
    pub spec_file: String
}
