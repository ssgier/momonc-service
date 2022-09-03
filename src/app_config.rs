use fxhash::FxHashMap;
pub const ADDR: &str = "127.0.0.1:8080";
pub const NUM_WORKER_THREADS: usize = 1;
pub type AppHashMap<K, V> = FxHashMap<K, V>;
