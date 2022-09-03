use crate::algo::AlgoConf;
use crate::obj_func::ObjFuncCallDef;
use crate::param::ParamsSpec;
use crate::processing;
use log::debug;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::task::JoinHandle;
use AppEvent::*;
use AppState::*;

#[derive(Debug)]
pub enum AppState {
    Idle,
    Processing(Option<JoinHandle<()>>),
    Terminal,
    Error,
}

#[derive(Debug)]
pub enum AppEvent {
    ProcessingJob(ParamsSpec, AlgoConf, ObjFuncCallDef),
    RequestStop,
}

#[derive(Debug)]
pub struct TransitionError(String);

impl AppState {
    pub fn new() -> AppState {
        AppState::Idle
    }

    fn transition_to(&mut self, new_state: AppState) {
        debug!("Transition to {:?}", new_state);
        *self = new_state;
    }

    pub async fn on_event(
        app_state: Arc<Mutex<Self>>,
        event: AppEvent,
    ) -> Result<(), TransitionError> {
        let mut state = app_state.lock().unwrap();
        match (&mut *state, event) {
            (Idle | Terminal, ProcessingJob(spec, algo_conf, obj_func_call_def)) => {
                let join_handle = tokio::spawn(processing::process(
                    spec,
                    algo_conf,
                    obj_func_call_def,
                    app_state.clone(),
                ));

                state.transition_to(Processing(Some(join_handle)));
                Ok(())
            }
            (Processing(join_handle_option), RequestStop) => {
                debug!("Stop requested");
                join_handle_option.take().unwrap().abort();

                // TODO: kill workers and think about awaiting result
                state.transition_to(Terminal);
                Ok(())
            }
            (_, event) => state.illegal_transition_error(event),
        }
    }

    fn illegal_transition_error(&self, event: AppEvent) -> Result<(), TransitionError> {
        Err(TransitionError(format!(
            "Event {:?} not allowed in state {:?}",
            event, *self
        )))
    }
}
