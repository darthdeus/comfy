#[derive(Copy, Clone, Debug)]
pub enum BurstState {
    Burst { remaining: i32, timer: f32 },
    Reload { timer: f32 },
    Idle,
}

#[derive(Copy, Clone, Debug)]
pub struct BurstTimer {
    pub reload_time: f32,

    pub burst_count: i32,
    pub burst_interval: f32,

    pub state: BurstState,
}

impl BurstTimer {
    pub fn new(
        reload_time: f32,
        burst_count: i32,
        burst_interval: f32,
    ) -> Self {
        Self {
            reload_time,

            burst_count,
            burst_interval,

            state: BurstState::Idle,
        }
    }

    pub fn try_fire(&mut self) -> bool {
        match self.state {
            BurstState::Burst { remaining, timer } => {
                if timer <= 0.0 {
                    if remaining > 0 {
                        self.state = BurstState::Burst {
                            remaining: remaining - 1,
                            timer: self.burst_interval,
                        };

                        true
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
            BurstState::Reload { .. } => false,
            BurstState::Idle => {
                self.state = BurstState::Burst {
                    remaining: self.burst_count - 1,
                    timer: self.burst_interval,
                };

                true
            }
        }
    }

    pub fn tick(&mut self, delta: f32) {
        match self.state {
            BurstState::Burst { remaining, timer } => {
                if timer > 0.0 {
                    self.state = BurstState::Burst {
                        remaining,
                        timer: timer - delta,
                    };
                } else if remaining == 0 {
                    self.state = BurstState::Reload {
                        timer: self.reload_time,
                    };
                }
            }
            BurstState::Reload { timer } => {
                if timer > 0.0 {
                    self.state = BurstState::Reload {
                        timer: timer - delta,
                    };
                } else {
                    self.state = BurstState::Idle;
                }
            }
            BurstState::Idle => {}
        }
    }
}
