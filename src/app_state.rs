use std::time::Instant;

use crate::algo::AlgoConf;
use crate::domain::DefaultProcessingJobData;
use crate::domain::DomainState;
use crate::domain::RequestMessage;
use crate::domain::StatusMessage;
use crate::obj_func::ObjFuncCallDef;
use crate::param::ParamsSpec;
use crate::processing;
use crate::processing_watcher::ProcessingWatcher;
use crate::type_aliases::EventReceiver;
use crate::type_aliases::EventSender;
use crate::type_aliases::StatusSender;
use log::debug;
use tokio::task::JoinHandle;
use AppEvent::*;
use DomainStateInner::*;

#[derive(Debug)]
pub enum DomainStateInner {
    Idle(DefaultProcessingJobData),
    Processing(Option<JoinHandle<()>>, ProcessingWatcher),
    Terminal,
    Error,
}

#[derive(Debug)]
pub enum AppEvent {
    NewSubscriber(StatusSender),
    ProcessingJob(ParamsSpec, AlgoConf, ObjFuncCallDef),
    Request(RequestMessage),
    RequestStop,
    DelegateStatusMessage(StatusMessage),
}

#[derive(Debug)]
pub struct TransitionError(String);

pub async fn run_app_fsm(
    mut recv: EventReceiver,
    sender: EventSender,
    default_processing_job_data: DefaultProcessingJobData,
) {
    let mut state = DomainStateInner::Idle(default_processing_job_data.clone());
    let mut subscriber: Option<StatusSender> = None;
    while let Some(event) = recv.recv().await {
        state = match (state, event) {
            (Idle(_) | Terminal, ProcessingJob(spec, algo_conf, obj_func_call_def)) => {
                let join_handle = tokio::spawn(processing::process(
                    spec,
                    algo_conf,
                    obj_func_call_def,
                    sender.clone(),
                ));

                let new_state =
                    Processing(Some(join_handle), ProcessingWatcher::new(Instant::now()));
                handle_subscription(&new_state, &mut subscriber);
                new_state
            }
            (state, AppEvent::NewSubscriber(new_subscriber)) => {
                subscriber = Some(new_subscriber);
                handle_subscription(&state, &mut subscriber);
                state
            }
            (mut state, AppEvent::DelegateStatusMessage(status_msg)) => {
                let current_time = Instant::now();

                if let Processing(_, processing_watcher) = &mut state {
                    processing_watcher.update(current_time);
                    processing_watcher.on_delegate_status_msg(&status_msg);
                }

                if let Some(subscriber_) = &mut subscriber {
                    subscriber_.send(status_msg).ok();
                }

                state
            }
            (Processing(mut join_handle_option, _), RequestStop) => {
                debug!("Stop requested");
                join_handle_option.take().unwrap().abort();
                // TODO: kill workers and think about awaiting result

                // TODO: transition to terminal
                let new_state = DomainStateInner::Idle(default_processing_job_data.clone());
                handle_subscription(&new_state, &mut subscriber);
                new_state
            }
            (state, event) => {
                debug!("Illegal event {:?} in state {:?}", event, state);
                state
            }
        }
    }
}

fn handle_subscription(domain_state: &DomainStateInner, subscriber: &mut Option<StatusSender>) {
    match subscriber {
        Some(subscriber_) => {
            let full_state = match domain_state {
                Idle(default_processing_job_data) => {
                    DomainState::Idle(default_processing_job_data.clone())
                }
                Processing(_, processing_watcher) => {
                    DomainState::Processing(processing_watcher.compute_processing_state())
                }
                Terminal => DomainState::Terminal,
                Error => DomainState::Error,
            };
            if subscriber_
                .send(StatusMessage::DomainState(full_state))
                .is_err()
            {
                *subscriber = None;
            }
        }
        _ => (),
    }
}
