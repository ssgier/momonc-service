use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum AlgoConf {
    ParallelHillClimbing(ParallelHillClimbingConf)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ParallelHillClimbingConf {
    pub std_dev: f64,
    pub degree_of_par: usize
}
