use std::{collections::VecDeque, time::Duration};

use crate::{
    app_config::CANDIDATE_WINDOW_SIZE_HINT,
    domain::{CandidateEvalReport, ProcessingState, StatusMessage},
    type_aliases::AppTime,
};

#[derive(Debug)]
pub struct ProcessingWatcher {
    pub start_time: AppTime,
    pub last_time: f64,
    eval_report_queue: VecDeque<CandidateEvalReport>,
}

impl ProcessingWatcher {
    pub fn new(time: AppTime) -> ProcessingWatcher {
        ProcessingWatcher {
            start_time: time,
            last_time: 0.0,
            eval_report_queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, time: AppTime) {
        self.last_time = time
            .duration_since(self.start_time)
            .unwrap_or(Duration::ZERO)
            .as_secs_f64();
    }

    pub fn on_delegate_status_msg(&mut self, message: &StatusMessage) {
        match message {
            StatusMessage::CandidateEvalReport(report) => {
                self.eval_report_queue.push_back(report.clone())
            }
            _ => (),
        };
    }

    pub fn compute_processing_state(&self) -> ProcessingState {
        ProcessingState {
            recent_candidate_eval_reports: self
                .eval_report_queue
                .iter()
                .map(|report| report.clone())
                .collect(),
            time: self.last_time,
            window_size_hint: CANDIDATE_WINDOW_SIZE_HINT,
        }
    }
}
