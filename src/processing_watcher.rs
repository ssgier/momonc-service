use crate::app_config::CANDIDATE_REPORT_REMEMBER_WINDOW_SECONDS;
use std::{collections::VecDeque, time::Instant};

use crate::domain::{CandidateEvalReport, ProcessingState, StatusMessage};

#[derive(Debug)]
pub struct ProcessingWatcher {
    start_time: Instant,
    last_time: f64,
    eval_report_queue: VecDeque<CandidateEvalReport>,
}

impl ProcessingWatcher {
    pub fn new(time: Instant) -> ProcessingWatcher {
        ProcessingWatcher {
            start_time: time,
            last_time: 0.0,
            eval_report_queue: VecDeque::new(),
        }
    }

    pub fn update(&mut self, time: Instant) {
        self.last_time = time.duration_since(self.start_time).as_secs_f64();
        let cut_off_time = self.last_time - CANDIDATE_REPORT_REMEMBER_WINDOW_SECONDS;
        loop {
            match self.eval_report_queue.front() {
                Some(head) if head.completion_time < cut_off_time => {
                    self.eval_report_queue.pop_front();
                }
                _ => break,
            }
        }
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
        }
    }
}
