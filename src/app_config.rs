use std::time::Duration;

pub const ADDR: &str = "127.0.0.1:3000";
pub const NUM_WORKER_THREADS: usize = 1;
pub const TIME_EVENT_INTERVAL: Duration = Duration::from_millis(250);
pub const CANDIDATE_WINDOW_LENGTH_HINT: usize = 250;
pub const BEST_SEEN_TABLE_SIZE_HINT: usize = 5;
