use crate::*;

use std::hash::Hash;

static NOTIFICATIONS: Lazy<AtomicRefCell<Notifications>> =
    Lazy::new(|| AtomicRefCell::new(Notifications::new()));

pub fn notifications() -> AtomicRefMut<'static, Notifications> {
    NOTIFICATIONS.borrow_mut()
}

pub struct Notification {
    pub text: String,
    pub timeout: Option<f32>,
    pub color: Color,
}

pub struct Notifications {
    pub notifications: Vec<Notification>,
}

impl Notifications {
    pub fn new() -> Self {
        Self { notifications: vec![] }
    }

    pub fn show(&mut self, text: &str, color: Color) {
        self.notifications.push(Notification {
            text: text.to_string(),
            timeout: Some(8.0),
            color,
        })
    }

    pub fn tick(&mut self, delta: f32) {
        self.notifications.retain_mut(|notification| {
            if let Some(ref mut time) = notification.timeout {
                *time -= delta;
                *time > 0.0
            } else {
                true
            }
        })
    }
}

pub struct ValueTracker {
    pub value: f32,
    pub last_value: f32,
    pub change_threshold: f32,
    pub time_since_last_update: f32,
    pub min_update_interval: f32,
}

impl ValueTracker {
    pub fn new(
        initial_value: f32,
        change_threshold: f32,
        min_update_interval: f32,
    ) -> Self {
        Self {
            value: initial_value,
            last_value: initial_value,
            change_threshold,
            time_since_last_update: 0.0,
            min_update_interval,
        }
    }

    pub fn update(&mut self, new_value: f32, dt: f32) -> bool {
        let change = (new_value - self.value).abs();

        self.time_since_last_update += dt;

        // Check if a large enough change has happened or if enough time has passed
        if change > self.change_threshold ||
            self.time_since_last_update > self.min_update_interval
        {
            self.last_value = self.value;
            self.value = new_value;
            self.time_since_last_update = 0.0;
            true
        } else {
            false
        }
    }

    pub fn get(&self) -> f32 {
        self.value
    }
}

static CHANGE_TRACKER: Lazy<AtomicRefCell<ChangeTracker>> =
    Lazy::new(|| AtomicRefCell::new(ChangeTracker::new()));

pub fn changes() -> AtomicRefMut<'static, ChangeTracker> {
    CHANGE_TRACKER.borrow_mut()
}

pub struct ChangeTracker {
    pub ints: HashMap<&'static str, i32>,
    pub floats: HashMap<&'static str, f32>,
    pub strings: HashMap<&'static str, String>,
}

impl ChangeTracker {
    pub fn new() -> Self {
        Self {
            ints: HashMap::default(),
            floats: HashMap::default(),
            strings: HashMap::default(),
        }
    }

    pub fn int(&mut self, key: &'static str, value: i32) -> bool {
        Self::value(&mut self.ints, key, value)
    }

    pub fn float(&mut self, key: &'static str, value: f32) -> bool {
        Self::value(&mut self.floats, key, value)
    }

    pub fn string(&mut self, key: &'static str, value: String) -> bool {
        Self::value(&mut self.strings, key, value)
    }

    fn value<T: PartialEq>(
        map: &mut HashMap<&'static str, T>,
        key: &'static str,
        value: T,
    ) -> bool {
        match map.entry(key) {
            Entry::Occupied(mut slot) => {
                if *slot.get() == value {
                    false
                } else {
                    slot.insert(value);
                    true
                }
            }
            Entry::Vacant(slot) => {
                slot.insert(value);
                true
            }
        }
    }
}

static COOLDOWNS: Lazy<AtomicRefCell<Cooldowns>> =
    Lazy::new(|| AtomicRefCell::new(Cooldowns::new()));

pub fn cooldowns() -> AtomicRefMut<'static, Cooldowns> {
    COOLDOWNS.borrow_mut()
}

pub struct Cooldowns {
    data: HashMap<u64, f32>,
}

impl Cooldowns {
    pub fn new() -> Self {
        Self { data: Default::default() }
    }

    pub fn tick(&mut self, delta: f32) {
        for (_, val) in self.data.iter_mut() {
            if *val > 0.0 {
                *val -= delta;
            }
        }
    }

    pub fn can_use<T: Hash>(&mut self, key: T, total: f32) -> bool {
        match self.data.entry(default_hash(&key)) {
            Entry::Occupied(mut slot) => {
                let result = *slot.get() <= 0.0;

                if result {
                    slot.insert(total);
                }

                result
            }
            Entry::Vacant(slot) => {
                slot.insert(total);
                true
            }
        }
    }

    pub fn can_use_random_not_first<T: Hash>(
        &mut self,
        key: T,
        total: f32,
        spread: f32,
    ) -> bool {
        match self.data.entry(default_hash(&key)) {
            Entry::Occupied(mut slot) => {
                let result = *slot.get() <= 0.0;

                if result {
                    let half = (spread * total) / 2.0;
                    slot.insert(total + gen_range(-half, half));
                }

                result
            }
            Entry::Vacant(slot) => {
                let half = (spread * total) / 2.0;
                slot.insert(total + gen_range(-half, half));
                false
            }
        }
    }
}
