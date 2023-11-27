use crate::*;

static PERF_COUNTERS: Lazy<AtomicRefCell<PerfCounters>> =
    Lazy::new(|| AtomicRefCell::new(PerfCounters::default()));

static TIMINGS: Lazy<AtomicRefCell<Timings>> =
    Lazy::new(|| AtomicRefCell::new(Timings::new()));

pub struct TimingEntry {
    pub history: egui::util::History<Duration>,
    pub time: Instant,
}

pub struct Timings {
    pub data: HashMap<&'static str, TimingEntry>,
}

impl Timings {
    pub fn new() -> Self {
        Self { data: HashMap::new() }
    }

    pub fn add_value(
        &mut self,
        name: &'static str,
        value: Duration,
        time: Instant,
    ) {
        let entry = self.data.entry(name).or_insert(TimingEntry {
            history: egui::util::History::new(50..2000, 2.0),
            time: Instant::now(),
        });

        entry.time = time;
        entry.history.add(get_time(), value);
    }

    pub fn span(&mut self, name: &'static str) -> TimingGuard<'_> {
        TimingGuard { timings: self, name, start: Instant::now() }
    }
}

pub struct TimingGuard<'a> {
    timings: &'a mut Timings,
    name: &'static str,
    start: Instant,
}

impl<'a> Drop for TimingGuard<'a> {
    fn drop(&mut self) {
        self.timings.add_value(self.name, self.start.elapsed(), self.start);
    }
}

pub struct AtomicTimingGuard {
    name: &'static str,
    start: Instant,
}

impl Drop for AtomicTimingGuard {
    fn drop(&mut self) {
        TIMINGS.borrow_mut().add_value(
            self.name,
            self.start.elapsed(),
            self.start,
        );
    }
}

pub fn timings() -> impl std::ops::Deref<Target = Timings> {
    TIMINGS.borrow_mut()
}

pub fn timing_start(name: &'static str) -> AtomicTimingGuard {
    AtomicTimingGuard { name, start: Instant::now() }
}

/// Add a timing value to a given timer, measured in f32 seconds.
pub fn timings_add_value(name: &'static str, value: f32) {
    TIMINGS.borrow_mut().add_value(
        name,
        Duration::from_secs_f32(value),
        Instant::now(),
    );
}

#[derive(Default)]
pub struct PerfCounters {
    // pub counters: HashMap<String, Counter>,
    pub counters: HashMap<Cow<'static, str>, Counter>,
}

#[derive(Default)]
pub struct Counter {
    pub count: u64,
    pub decayed_average: f64,
}

impl PerfCounters {
    pub fn global() -> AtomicRef<'static, PerfCounters> {
        PERF_COUNTERS.borrow()
    }

    // pub fn update_counter(&mut self, counter_name: &str, count: u64) {
    //     let counter =
    //         self.counters.entry(counter_name.to_string()).or_default();
    //     counter.count = count;
    // }
    pub fn update_counter(
        &mut self,
        counter_name: impl Into<Cow<'static, str>>,
        count: u64,
    ) {
        let counter = self.counters.entry(counter_name.into()).or_default();
        counter.count = count;
    }

    pub fn new_frame(&mut self, delta: f64) {
        for counter in self.counters.values_mut() {
            counter.decayed_average = counter.decayed_average * (1.0 - delta) +
                (counter.count as f64) * delta;
            counter.count = 0;
        }
    }

    pub fn get_counter(&self, counter_name: &str) -> (u64, f64) {
        if let Some(counter) = self.counters.get(counter_name) {
            (counter.count, counter.decayed_average)
        } else {
            (0, 0.0)
        }
    }

    pub fn reset_counters(&mut self) {
        self.counters.clear();
    }
}

pub fn perf_counters_new_frame(delta: f64) {
    let mut counters = PERF_COUNTERS.borrow_mut();
    counters.new_frame(delta);
}

pub fn reset_perf_counters() {
    let mut counters = PERF_COUNTERS.borrow_mut();
    counters.reset_counters();
}

pub fn perf_counter(counter_name: impl Into<Cow<'static, str>>, count: u64) {
    let mut counters = PERF_COUNTERS.borrow_mut();
    counters.update_counter(counter_name, count);
}

pub fn perf_counter_inc(counter_name: impl Into<Cow<'static, str>>, inc: u64) {
    let mut counters = PERF_COUNTERS.borrow_mut();
    let counter_name_cow = counter_name.into();
    let (current_value, _) = counters.get_counter(&counter_name_cow);
    counters.update_counter(counter_name_cow, current_value + inc);
}

pub fn get_perf_counter(
    counter_name: impl Into<Cow<'static, str>>,
) -> (u64, f64) {
    let counters = PERF_COUNTERS.borrow_mut();
    counters.get_counter(&counter_name.into())
}

// pub fn perf_counter(counter_name: &str, count: u64) {
//     let mut counters = PERF_COUNTERS.borrow_mut();
//     counters.update_counter(counter_name, count);
// }
//
// pub fn perf_counter_inc(counter_name: &str) {
//     let mut counters = PERF_COUNTERS.borrow_mut();
//     let (current_value, _) = counters.get_counter(counter_name);
//     counters.update_counter(counter_name, current_value + 1);
// }
//
// pub fn get_perf_counter(counter_name: &str) -> (u64, f64) {
//     let counters = PERF_COUNTERS.borrow_mut();
//     counters.get_counter(counter_name)
// }
