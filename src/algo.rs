use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AlgoConf {
    ParallelHillClimbing(ParallelHillClimbingConf)
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ParallelHillClimbingConf {
    pub relative_std_dev: f64,
    pub degree_of_par: usize
}
