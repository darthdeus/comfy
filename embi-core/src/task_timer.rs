use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

pub struct TaskTimer {
    timers: Arc<Mutex<HashMap<String, Duration>>>,
}

impl TaskTimer {
    fn new() -> Self {
        TaskTimer { timers: Arc::new(Mutex::new(HashMap::new())) }
    }

    fn start_task(&self, task_name: &str) -> TaskGuard {
        let task_name = task_name.to_string();
        let timers = Arc::clone(&self.timers);
        TaskGuard { task_name, timers, start_time: Instant::now() }
    }

    fn get_duration(&self, task_name: &str) -> Option<Duration> {
        let timers = self.timers.lock().unwrap();
        timers.get(task_name).cloned()
    }
}

pub struct TaskGuard {
    task_name: String,
    timers: Arc<Mutex<HashMap<String, Duration>>>,
    start_time: Instant,
}

impl Drop for TaskGuard {
    fn drop(&mut self) {
        let elapsed = self.start_time.elapsed();
        let mut timers = self.timers.lock().unwrap();
        let entry =
            timers.entry(self.task_name.clone()).or_insert(Duration::new(0, 0));
        *entry += elapsed;
    }
}

lazy_static! {
    static ref GLOBAL_TASK_TIMER: TaskTimer = TaskTimer::new();
}

pub fn start_task(task_name: &str) -> TaskGuard {
    GLOBAL_TASK_TIMER.start_task(task_name)
}

pub fn get_duration(task_name: &str) -> Option<Duration> {
    GLOBAL_TASK_TIMER.get_duration(task_name)
}
