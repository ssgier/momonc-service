
#[derive(Debug)]
pub enum AlgoConf {
    ParallelHillClimbing(ParallelHillClimbingConf)
}

#[derive(Debug)]
pub struct ParallelHillClimbingConf {
    pub std_dev: f64,
    pub num_values_per_iter: usize
}
