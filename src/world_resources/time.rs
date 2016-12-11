use std::time::{Duration, Instant};

pub struct Time {
    pub delta_time: Duration,
    pub fixed_step: Duration,
    pub last_fixed_update: Instant,
}
