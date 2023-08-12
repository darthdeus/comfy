use crate::*;

pub struct Tween {
    initial_val: f32,
    final_val: f32,
    duration: f32,
    elapsed: f32,
    delay: f32,
    easing_fn: fn(f32) -> f32,
    value: f32,
}

impl Default for Tween {
    fn default() -> Self {
        Self::new(0.0, 0.0, 0.0, 0.0, linear)
    }
}

impl Tween {
    pub fn new(
        initial_val: f32,
        final_val: f32,
        duration: f32,
        delay: f32,
        easing_fn: fn(f32) -> f32,
    ) -> Self {
        Tween {
            initial_val,
            final_val,
            duration,
            elapsed: 0.0,
            delay,
            easing_fn,
            value: initial_val,
        }
    }

    pub fn is_finished(&self) -> bool {
        self.elapsed >= self.duration + self.delay
    }

    pub fn update(&mut self, delta_time: f32) {
        if self.delay > 0.0 {
            self.delay -= delta_time;
            return;
        }

        self.elapsed += delta_time;
        let progress = (self.elapsed / self.duration).min(1.0);

        let t = (self.easing_fn)(progress);
        self.value = self.initial_val + t * (self.final_val - self.initial_val);
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

#[derive(Copy, Clone, Debug)]
pub struct FlashingColor {
    color: Color,
    flash_color: Color,
    duration: f32,
    total_remaining: f32,
    interval: f32,
    interval_remaining: f32,
    easing_fn: fn(f32) -> f32,
}

impl FlashingColor {
    pub fn new(
        color: Color,
        flash_color: Color,
        duration: f32,
        interval: f32,
        easing_fn: fn(f32) -> f32,
    ) -> Self {
        Self {
            color,
            flash_color,
            duration,
            total_remaining: 0.0,
            interval,
            interval_remaining: 0.0,
            easing_fn,
        }
    }

    pub fn trigger(&mut self) {
        self.total_remaining = self.duration;
    }

    pub fn update(&mut self, delta: f32) {
        if self.total_remaining > 0.0 {
            self.total_remaining -= delta;
            self.interval_remaining -= delta;

            if self.interval_remaining <= 0.0 {
                self.interval_remaining = self.interval;
            }
        }
    }

    pub fn current_color(&self) -> Color {
        if self.total_remaining <= 0.0 {
            self.color
        } else {
            let progress = (self.interval_remaining / self.interval).min(1.0);
            let t = (self.easing_fn)(progress);
            Color::lerp(self.color, self.flash_color, t)
        }
    }
}
