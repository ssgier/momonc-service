use std::{collections::VecDeque, time::Duration};

use crate::{
    app_config::{BEST_SEEN_TABLE_SIZE_HINT, CANDIDATE_WINDOW_LENGTH_HINT},
    domain::{CandidateEvalReport, ProcessingState, StatusMessage},
    type_aliases::AppTime,
};

#[derive(Debug)]
pub struct ProcessingWatcher {
    pub start_time: AppTime,
    pub last_time: f64,
    eval_report_queue: VecDeque<CandidateEvalReport>,
    best_seen_reports: Vec<CandidateEvalReport>,
}

impl ProcessingWatcher {
    pub fn new(time: AppTime) -> ProcessingWatcher {
        ProcessingWatcher {
            start_time: time,
            last_time: 0.0,
            eval_report_queue: VecDeque::new(),
            best_seen_reports: Vec::with_capacity(BEST_SEEN_TABLE_SIZE_HINT),
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
                self.eval_report_queue.push_back(report.clone());

                if let Some(obj_func_val) = report.obj_func_val {
                    if self.best_seen_reports.len() < BEST_SEEN_TABLE_SIZE_HINT
                        || obj_func_val
                            < self.best_seen_reports.last().unwrap().obj_func_val.unwrap()
                    {
                        self.best_seen_reports.push(report.clone());
                        self.best_seen_reports.sort_by(|a, b| {
                            a.obj_func_val
                                .unwrap()
                                .partial_cmp(&b.obj_func_val.unwrap())
                                .unwrap()
                        });
                        if self.best_seen_reports.len() > BEST_SEEN_TABLE_SIZE_HINT {
                            self.best_seen_reports.pop();
                        }
                    }
                }
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
            best_seen_candidate_eval_reports: self.best_seen_reports.clone(),
            time: self.last_time,
            window_length_hint: CANDIDATE_WINDOW_LENGTH_HINT,
            best_seen_table_size_hint: BEST_SEEN_TABLE_SIZE_HINT,
        }
    }
}
