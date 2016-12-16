//! This module contains `Time` struct, which holds information
//! about time which has elapsed since the previous frame.

use std::time::{Duration, Instant};

/// `Time` is added to `ecs::World` as a resource by default.
/// It is updated every frame in `Application::advance_frame`.
pub struct Time {
    /// Time elapsed since the last frame.
    pub delta_time: Duration,
    /// Rate at which `State::fixed_update` is called.
    pub fixed_step: Duration,
    /// Time at which `State::fixed_update` was last called.
    pub last_fixed_update: Instant,
}
