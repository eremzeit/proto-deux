use std::{
    collections::HashMap,
    sync::Mutex,
    time::{Duration, Instant},
};

use once_cell::sync::Lazy;

// pub struct TimerEntry {
//     pub is_started: bool,
//     pub start_time: Instant,
// }

pub struct TimerCache {
    pub entries: HashMap<String, Instant>,
    pub counts: HashMap<String, Duration>,
}

impl TimerCache {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
            counts: HashMap::new(),
        }
    }
    pub fn start(&mut self, key: &str) {
        self.entries.insert(key.to_owned(), Instant::now());
        if !self.counts.contains_key(key) {
            self.counts.insert(key.to_owned(), Duration::new(0, 0));
        }
    }

    pub fn stop(&mut self, key: &str) {
        let d = self.entries.remove(key).unwrap().elapsed();
        let prev = self.counts.get(key).unwrap().to_owned();
        self.counts.insert(key.to_owned(), prev + d);
    }

    pub fn print(&self) {}
}

pub static PERF_TIMER: Lazy<Mutex<TimerCache>> = Lazy::new(|| Mutex::new(TimerCache::new()));

#[macro_export]
macro_rules! perf_timer_start {
    ($x:expr) => {{
        if cfg!(not(test)) {
            use crate::perf::perf_timer_start;
            perf_timer_start($x);
        }
    }};
}

#[macro_export]
macro_rules! perf_timer_stop {
    ($x:expr) => {{
        if cfg!(not(test)) {
            use crate::perf::perf_timer_stop;
            perf_timer_stop($x);
        }
    }};
}

#[macro_export]
macro_rules! perf_timers_print {
    () => {{
        if cfg!(not(test)) {
            use crate::perf::perf_timer_print;
            perf_timer_print();
        }
    }};
}

pub fn perf_timer_start(key: &str) {
    PERF_TIMER.lock().unwrap().start(key);
}

pub fn perf_timer_stop(key: &str) {
    PERF_TIMER.lock().unwrap().stop(key);
}

pub fn perf_timer_print() {
    let timers = PERF_TIMER.lock().unwrap();
    if !timers.counts.is_empty() {
        println!("PERF_TIMERS:");
        for (key, val) in timers.counts.iter() {
            println!("{key}: {}ms", val.as_millis())
        }
    }
}
