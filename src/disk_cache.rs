use std::path::Path;

use crate::domain::ProcessingJobData;
use crate::{
    algo::{AlgoConf, ParallelHillClimbingConf},
    domain::DefaultProcessingJobData,
};
use home;

pub fn retrieve_default_processing_job_data() -> DefaultProcessingJobData {
    gen_init_default_proc_job_data()
}

pub fn store_default_processing_job_data(_data: &DefaultProcessingJobData) {}

fn gen_abs_path_as_string(rel_path_from_home: &Path) -> String {
    let mut home_dir = home::home_dir().expect("Unable to determine home directory");
    home_dir.push(rel_path_from_home);
    home_dir.into_os_string().into_string().unwrap()
}

fn gen_init_default_proc_job_data() -> DefaultProcessingJobData {
    DefaultProcessingJobData(ProcessingJobData {
        spec_file: gen_abs_path_as_string(Path::new("git/momonc-service/scripts/spec.json")),
        program: "python".to_string(),
        args: vec![gen_abs_path_as_string(Path::new(
            "git/momonc-service/scripts/obj_func_mock.py",
        ))],
        algo_conf: AlgoConf::ParallelHillClimbing(ParallelHillClimbingConf {
            relative_std_dev: 0.01,
            degree_of_par: 10,
        }),
    })
}
