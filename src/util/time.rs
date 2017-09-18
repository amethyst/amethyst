//! Util time conversion functions
use std::time::Duration;
///Converts a Duration to the time in seconds.
pub fn duration_to_secs(duration: &Duration) -> f32 {
    duration.as_secs() as f32 + (duration.subsec_nanos() as f32 / 1.0e9)
}
///Converts a Duration to nanoseconds
pub fn duration_to_nanos(duration: &Duration) -> u64 {
    (duration.as_secs() * 1_000_000_000) + duration.subsec_nanos() as u64
}
