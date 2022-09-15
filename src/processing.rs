use crate::app_state::AppEvent;
use crate::domain::{CandidateEvalReport, StatusMessage};
use crate::obj_func::{self, ObjFuncCallDef};
use crate::param::Dim;
use crate::type_aliases::EventSender;
use futures::future;
use log::debug;
use rand::SeedableRng;
use rand::{
    distributions::{Bernoulli, Distribution},
    rngs::StdRng,
};
use rand_distr::Normal;
use serde_json::Number as NumberValue;
use serde_json::Value::{Bool, Number, Object};
use std::sync::Arc;
use std::sync::Mutex;

use crate::algo::{
    AlgoConf::{self, *},
    ParallelHillClimbingConf,
};
use crate::param::{ParamsSpec, ParamsValue};
use std::time::Instant;

pub async fn process(
    spec: ParamsSpec,
    algo_conf: AlgoConf,
    obj_func_call_def: ObjFuncCallDef,
    event_sender: EventSender,
) {
    match algo_conf {
        ParallelHillClimbing(parallel_hill_climbing_conf) => {
            parallel_hill_climbing(
                spec,
                parallel_hill_climbing_conf,
                obj_func_call_def,
                event_sender,
            )
            .await;
        }
    }
}

#[derive(Debug)]
struct BestSeen {
    candidate: serde_json::Value,
    obj_func_val: f64,
}

type BestSeenContext = Arc<Mutex<Option<BestSeen>>>;

async fn parallel_hill_climbing(
    spec: ParamsSpec,
    algo_conf: ParallelHillClimbingConf,
    obj_func_call_def: ObjFuncCallDef,
    event_sender: EventSender,
) {
    let initial_guess = Object(spec.extract_initial_guess());
    let best_seen: BestSeenContext = Arc::new(Mutex::new(None));

    debug!("Starting with initial guess: {:?}", &initial_guess);
    let mut rng = StdRng::seed_from_u64(0);
    let processing_start_instant = Instant::now();

    for iter_num in 0.. {
        let candidates: Vec<serde_json::Value> = (0..algo_conf.degree_of_par)
            .map(|candidate_number| {
                if iter_num == 0 && candidate_number == 0 {
                    initial_guess.clone()
                } else {
                    let best_seen_option = best_seen.lock().unwrap();
                    let from_candidate = best_seen_option
                        .as_ref()
                        .map(|best| &best.candidate)
                        .unwrap_or(&initial_guess)
                        .as_object()
                        .unwrap();

                    Object(create_candidate(
                        from_candidate,
                        &spec,
                        &algo_conf,
                        &mut rng,
                    ))
                }
            })
            .collect();

        let iteration_start_time = processing_start_instant.elapsed().as_secs_f64();

        let eval_candidate_futures = candidates.into_iter().map(|candidate| {
            evaluate_candidate_and_report(
                &obj_func_call_def,
                candidate,
                best_seen.clone(),
                &processing_start_instant,
                iteration_start_time,
                event_sender.clone(),
            )
        });

        future::join_all(eval_candidate_futures).await;

        debug!(
            "Iteration {} completed. Best seen: {:?}",
            iter_num, best_seen
        );
    }
}

async fn evaluate_candidate_and_report(
    obj_func_call_def: &ObjFuncCallDef,
    new_candidate: serde_json::Value,
    best_seen_context: BestSeenContext,
    processing_start_instant: &Instant,
    iteration_start_time: f64,
    event_sender: EventSender,
) {
    let new_obj_func_val_option = obj_func::call(obj_func_call_def, &new_candidate).await;
    let completion_time = processing_start_instant.elapsed().as_secs_f64();

    let mut best_seen_option = best_seen_context.lock().unwrap();
    let obj_func_val_before = best_seen_option
        .as_ref()
        .map(|best_seen| best_seen.obj_func_val);

    match new_obj_func_val_option {
        Some(new_obj_func_val) => {
            let replace = best_seen_option
                .as_ref()
                .map(|best_seen| new_obj_func_val < best_seen.obj_func_val)
                .unwrap_or(true);

            if replace {
                *best_seen_option = Some(BestSeen {
                    candidate: new_candidate.clone(),
                    obj_func_val: new_obj_func_val,
                });
            }
        }
        None => (),
    };

    let report = CandidateEvalReport {
        start_time: iteration_start_time,
        completion_time,
        obj_func_val: new_obj_func_val_option,
        best_seen_obj_func_val_before: obj_func_val_before,
        candidate: new_candidate.clone(),
    };

    event_sender
        .send(AppEvent::DelegateStatusMessage(
            StatusMessage::CandidateEvalReport(report),
        ))
        .ok();
}

fn create_candidate(
    from_candidate: &ParamsValue,
    params_spec: &ParamsSpec,
    conf: &ParallelHillClimbingConf,
    rng: &mut StdRng,
) -> ParamsValue {
    let mut result = ParamsValue::default();
    let std_dev = conf.relative_std_dev;
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
