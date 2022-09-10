use crate::algo::AlgoConf;
use crate::domain::DefaultProcessingJobData;
use crate::domain::DomainState;
use crate::domain::RequestMessage;
use crate::domain::StatusMessage;
use crate::obj_func::ObjFuncCallDef;
use crate::param::ParamsSpec;
use crate::processing;
use crate::type_aliases::EventReceiver;
use crate::type_aliases::EventSender;
use crate::type_aliases::Subscriber;
use log::debug;
use tokio::task::JoinHandle;
use AppEvent::*;
use DomainStateInner::*;

#[derive(Debug)]
pub enum DomainStateInner {
    Idle(DefaultProcessingJobData),
    Processing(Option<JoinHandle<()>>),
    Terminal,
    Error,
}

#[derive(Debug)]
pub enum AppEvent {
    NewSubscriber(Subscriber),
    ProcessingJob(ParamsSpec, AlgoConf, ObjFuncCallDef),
    Request(RequestMessage),
    RequestStop,
}

#[derive(Debug)]
pub struct TransitionError(String);

pub async fn run_app_fsm(
    mut recv: EventReceiver,
    sender: EventSender,
    default_processing_job_data: DefaultProcessingJobData,
) {
    let mut state = DomainStateInner::Idle(default_processing_job_data);
    let mut subscribers = Vec::new();
    while let Some(event) = recv.recv().await {
        state = match (state, event) {
            (Idle(_) | Terminal, ProcessingJob(spec, algo_conf, obj_func_call_def)) => {
                let join_handle = tokio::spawn(processing::process(
                    spec,
                    algo_conf,
                    obj_func_call_def,
                    sender.clone(),
                ));

                Processing(Some(join_handle))
            }
            (state, AppEvent::NewSubscriber(subscriber)) => {
                if subscriber
                    .send(StatusMessage::DomainState(get_full_state(&state)))
                    .is_ok()
                {
                    subscribers.push(subscriber);
                }
                state
            }
            (Processing(mut join_handle_option), RequestStop) => {
                debug!("Stop requested");
                join_handle_option.take().unwrap().abort();

                // TODO: kill workers and think about awaiting result
                Terminal
            }
            (state, event) => {
                debug!("Illegal event {:?} in state {:?}", event, state);
                state
            }
        }
    }
}

fn get_full_state(domain_state: &DomainStateInner) -> DomainState {
    match domain_state {
        Idle(default_processing_job_data) => DomainState::Idle(default_processing_job_data.clone()),
        Processing(_) => DomainState::Processing,
        Terminal => DomainState::Terminal,
        Error => DomainState::Error,
    }
}
