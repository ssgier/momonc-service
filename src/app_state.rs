use crate::algo::AlgoConf;
use crate::app_config::TIME_EVENT_INTERVAL;
use crate::domain::DefaultProcessingJobData;
use crate::domain::DomainState;
use crate::domain::RequestMessage;
use crate::domain::StatusMessage;
use crate::obj_func::ObjFuncCallDef;
use crate::param::ParamsSpec;
use crate::processing;
use crate::processing_watcher::ProcessingWatcher;
use crate::type_aliases::{AppTime, EventReceiver, EventSender, StatusSender};
use log::debug;
use std::time::Duration;
use tokio::{task::JoinHandle, time};
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
    PublishTime,
    RequestStop,
    DelegateStatusMessage(StatusMessage),
}

#[derive(Debug)]
pub struct TransitionError(String);

pub async fn run_app_fsm(
    mut recv: EventReceiver,
    event_sender: EventSender,
    default_processing_job_data: DefaultProcessingJobData,
) {
    let mut state = DomainStateInner::Idle(default_processing_job_data.clone());
    let mut subscriber: Option<StatusSender> = None;

    schedule_time_events(event_sender.clone());

    while let Some(event) = recv.recv().await {
        state = match (state, event) {
            (Idle(_) | Terminal, ProcessingJob(spec, algo_conf, obj_func_call_def)) => {
                let processing_start_instant = AppTime::now();
                let join_handle = tokio::spawn(processing::process(
                    processing_start_instant,
                    spec,
                    algo_conf,
                    obj_func_call_def,
                    event_sender.clone(),
                ));

                let new_state = Processing(
                    Some(join_handle),
                    ProcessingWatcher::new(processing_start_instant),
                );
                handle_subscription(&new_state, &mut subscriber);
                new_state
            }
            (state, AppEvent::NewSubscriber(new_subscriber)) => {
                subscriber = Some(new_subscriber);
                handle_subscription(&state, &mut subscriber);
                state
            }
            (mut state, AppEvent::DelegateStatusMessage(status_msg)) => {
                let current_time = AppTime::now();

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
            (mut state, AppEvent::PublishTime) => {
                match (&mut state, &subscriber) {
                    (Processing(_, processing_watcher), Some(subscriber_)) => {
                        processing_watcher.update(AppTime::now());
                        subscriber_
                            .send(StatusMessage::Time(
                                processing_watcher
                                    .start_time
                                    .elapsed()
                                    .unwrap_or(Duration::ZERO)
                                    .as_secs_f64(),
                            ))
                            .ok();
                        debug!(
                            "{}",
                            processing_watcher
                                .start_time
                                .elapsed()
                                .unwrap_or(Duration::ZERO)
                                .as_secs_f64()
                        );
                    }
                    _ => (),
                }
                state
            }
            (state, event) => {
                debug!("Illegal event {:?} in state {:?}", event, state);
                state
            }
        }
    }
}

fn schedule_time_events(event_sender: EventSender) {
    tokio::spawn(async move {
        let mut interval = time::interval(TIME_EVENT_INTERVAL);
        loop {
            interval.tick().await;
            if let Err(_) = event_sender.send(AppEvent::PublishTime) {
                break;
            }
        }
    });
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
