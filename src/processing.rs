use crate::obj_func::{self, ObjFuncCallDef};
use crate::param::Dim;
use futures::future;
use log::debug;
use rand::SeedableRng;
use rand::{
    distributions::{Bernoulli, Distribution},
    rngs::StdRng,
};
use rand_distr::Normal;
use serde_json::Number as NumberValue;
use serde_json::Value::{self, Bool, Number, Object};
use std::sync::{Arc, Mutex};

use crate::algo::{
    AlgoConf::{self, *},
    ParallelHillClimbingConf,
};
use crate::app_state::AppState;
use crate::param::{ParamsSpec, ParamsValue};

pub async fn process(
    spec: ParamsSpec,
    algo_conf: AlgoConf,
    obj_func_call_def: ObjFuncCallDef,
    app_state: Arc<Mutex<AppState>>,
) {
    match algo_conf {
        ParallelHillClimbing(parallel_hill_climbing_conf) => {
            parallel_hill_climbing(
                spec,
                parallel_hill_climbing_conf,
                obj_func_call_def,
                app_state,
            )
            .await;
        }
    }
}

async fn parallel_hill_climbing(
    spec: ParamsSpec,
    algo_conf: ParallelHillClimbingConf,
    obj_func_call_def: ObjFuncCallDef,
    _app_state: Arc<Mutex<AppState>>,
) {
    let mut current_value = Object(spec.extract_initial_guess());
    let mut current_obj_func_val = f64::MAX;

    debug!("Starting with initial guess: {:?}", &current_value);
    let mut rng = StdRng::seed_from_u64(0);

    for iter_num in 0.. {
        let candidates: Vec<Value> = (0..algo_conf.degree_of_par)
            .map(|candidate_number| {
                if iter_num == 0 && candidate_number == 0 {
                    current_value.clone()
                } else {
                    Object(create_candidate(
                        &current_value.as_object().unwrap(),
                        &spec,
                        &algo_conf,
                        &mut rng,
                    ))
                }
            })
            .collect();

        let eval_candidate_futures = candidates
            .into_iter()
            .map(|candidate| evaluate_candidate(&obj_func_call_def, candidate));

        let best_candidate = future::join_all(eval_candidate_futures)
            .await
            .into_iter()
            .min_by(|x, y| x.0.partial_cmp(&y.0).unwrap())
            .unwrap();

        if best_candidate.0 < current_obj_func_val {
            current_value = best_candidate.1;
            current_obj_func_val = best_candidate.0;
        }

        debug!(
            "Iteration {} completed. Objective function value: {}",
            iter_num, current_obj_func_val
        );
    }
}

async fn evaluate_candidate(obj_func_call_def: &ObjFuncCallDef, candidate: Value) -> (f64, Value) {
    let obj_func_val = obj_func::call(obj_func_call_def, &candidate).await;
    (obj_func_val, candidate)
}

fn create_candidate(
    from_candidate: &ParamsValue,
    params_spec: &ParamsSpec,
    conf: &ParallelHillClimbingConf,
    rng: &mut StdRng,
) -> ParamsValue {
    let mut result = ParamsValue::default();
    let std_dev = conf.std_dev;
    for dim_spec in &params_spec.dims {
        match dim_spec {
            Dim::Boolean(bool_spec) => {
                let from_value = from_candidate
                    .get(&bool_spec.name)
                    .unwrap()
                    .as_bool()
                    .unwrap();
                let sample = Bernoulli::new(std_dev.min(1.0)).unwrap().sample(rng);
                let result_value = sample ^ from_value;
                result.insert(bool_spec.name.clone(), Bool(result_value));
            }
            Dim::RealNumber(real_num_spec) => {
                let from_value = from_candidate
                    .get(&real_num_spec.dim_spec.name)
                    .unwrap()
                    .as_f64()
                    .unwrap();
                let stdev_to_use =
                    std_dev * (real_num_spec.max_value_excl - real_num_spec.min_value_incl);
                let result_value = Normal::new(from_value, stdev_to_use).unwrap().sample(rng);
                let result_value = result_value
                    .min(real_num_spec.max_value_excl)
                    .max(real_num_spec.min_value_incl);
                result.insert(
                    real_num_spec.dim_spec.name.clone(),
                    Number(NumberValue::from_f64(result_value).unwrap()),
                );
            }
            Dim::Integer(int_spec) => {
                let from_value = from_candidate
                    .get(&int_spec.dim_spec.name)
                    .unwrap()
                    .as_i64()
                    .unwrap();
                let diff = int_spec.max_value_excl - int_spec.min_value_incl;
                let stdev_to_use = std_dev * (diff as f64);
                let result_value = Normal::new(from_value as f64, stdev_to_use)
                    .unwrap()
                    .sample(rng);
                let result_value = (result_value as i64)
                    .min(int_spec.max_value_excl)
                    .max(int_spec.min_value_incl);
                result.insert(
                    int_spec.dim_spec.name.clone(),
                    Number(NumberValue::from(result_value)),
                );
            }
        }
    }
    result
}
