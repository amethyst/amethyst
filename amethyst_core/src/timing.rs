//! Utilities for working with time.

use std::time::{Duration, Instant};

/// Frame timing values.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Time {
    /// Time elapsed since the last frame in seconds.
    delta_seconds: f32,
    /// Time elapsed since the last frame.
    delta_time: Duration,
    /// Time elapsed since the last frame in seconds ignoring the time speed multiplier.
    delta_real_seconds: f32,
    /// Time elapsed since the last frame ignoring the time speed multiplier.
    delta_real_time: Duration,
    /// Rate at which `State::fixed_update` is called in seconds.
    fixed_seconds: f32,
    /// Rate at which `State::fixed_update` is called.
    fixed_time: Duration,
    /// The total number of frames that have been played in this session.
    frame_number: u64,
    ///Time elapsed since game start, ignoring the speed multipler.
    absolute_real_time: Duration,
    ///Time elapsed since game start, taking the speed multiplier into account.
    absolute_time: Duration,
    ///Time multiplier. Affects returned delta_seconds, delta_time and absolute_time.
    time_scale: f32,
    /// Fixed timestep accumulator.
    fixed_time_accumulator: f32,
    /// Fixed update interpolation alpha
    interpolation_alpha: f32,
}

impl Time {
    /// Gets the time difference between frames in seconds.
    ///
    /// This function should not be used during `fixed_update`s, use `fixed_seconds` instead.
    pub fn delta_seconds(&self) -> f32 {
        self.delta_seconds
    }

    /// Gets the time difference between frames.
    ///
    /// This function should not be used during `fixed_update`s, use `fixed_time` instead.
    pub fn delta_time(&self) -> Duration {
        self.delta_time
    }

    /// Gets the time difference between frames in seconds ignoring the time speed multiplier.
    ///
    /// This function should not be used during `fixed_update`s.
    pub fn delta_real_seconds(&self) -> f32 {
        self.delta_real_seconds
    }

    /// Gets the time difference between frames ignoring the time speed multiplier.
    pub fn delta_real_time(&self) -> Duration {
        self.delta_real_time
    }

    /// Gets the fixed time step in seconds.
    pub fn fixed_seconds(&self) -> f32 {
        self.fixed_seconds
    }

    /// Gets the fixed time step.
    pub fn fixed_time(&self) -> Duration {
        self.fixed_time
    }

    /// Gets the current frame number.  This increments by 1 every frame.  There is no frame 0.
    pub fn frame_number(&self) -> u64 {
        self.frame_number
    }

    /// Gets the time since the start of the game, taking into account the speed multiplier.
    pub fn absolute_time(&self) -> Duration {
        self.absolute_time
    }

    /// Gets the time since the start of the game as seconds, taking into account the speed multiplier.
    pub fn absolute_time_seconds(&self) -> f64 {
        duration_to_secs_f64(self.absolute_time)
    }

    /// Gets the time since the start of the game, ignoring the speed multiplier.
    pub fn absolute_real_time(&self) -> Duration {
        self.absolute_real_time
    }

    /// Gets the time since the start of the game as seconds, ignoring the speed multiplier.
    pub fn absolute_real_time_seconds(&self) -> f64 {
        duration_to_secs_f64(self.absolute_real_time)
    }

    /// Gets the current time speed multiplier.
    pub fn time_scale(&self) -> f32 {
        self.time_scale
    }

    /// Gets the current interpolation alpha factor.
    pub fn interpolation_alpha(&self) -> f32 {
        self.interpolation_alpha
    }

    /// Sets both `delta_seconds` and `delta_time` based on the seconds given.
    ///
    /// This should only be called by the engine.  Bad things might happen if you call this in
    /// your game.
    pub fn set_delta_seconds(&mut self, secs: f32) {
        self.delta_seconds = secs * self.time_scale;
        self.delta_time = secs_to_duration(secs * self.time_scale);
        self.delta_real_seconds = secs;
        self.delta_real_time = secs_to_duration(secs);

        self.absolute_time += self.delta_time;
        self.absolute_real_time += self.delta_real_time;
    }

    /// Sets both `delta_time` and `delta_seconds` based on the duration given.
    ///
    /// This should only be called by the engine.  Bad things might happen if you call this in
    /// your game.
    pub fn set_delta_time(&mut self, time: Duration) {
        self.delta_seconds = duration_to_secs(time) * self.time_scale;
        self.delta_time = secs_to_duration(duration_to_secs(time) * self.time_scale);
        self.delta_real_seconds = duration_to_secs(time);
        self.delta_real_time = time;

        self.absolute_time += self.delta_time;
        self.absolute_real_time += self.delta_real_time;
    }

    /// Sets both `fixed_seconds` and `fixed_time` based on the seconds given.
    pub fn set_fixed_seconds(&mut self, secs: f32) {
        self.fixed_seconds = secs;
        self.fixed_time = secs_to_duration(secs);
    }

    /// Sets both `fixed_time` and `fixed_seconds` based on the duration given.
    pub fn set_fixed_time(&mut self, time: Duration) {
        self.fixed_seconds = duration_to_secs(time);
        self.fixed_time = time;
    }

    /// Increments the current frame number by 1.
    ///
    /// This should only be called by the engine.  Bad things might happen if you call this in
    /// your game.
    pub fn increment_frame_number(&mut self) {
        self.frame_number += 1;
    }

    /// Sets the time multiplier that affects how time values are computed,
    /// effectively slowing or speeding up your game.
    ///
    /// ## Panics
    /// This will panic if multiplier is NaN, Infinity, or less than 0.
    pub fn set_time_scale(&mut self, multiplier: f32) {
        use std::f32::INFINITY;
        assert!(multiplier >= 0.0);
        assert!(multiplier != INFINITY);
        self.time_scale = multiplier;
    }

    /// Restarts the internal fixed update accumulator to the desired fixed update delta time.
    ///
    /// This should only be called by the engine.  Bad things might happen if you call this in
    /// your game.
    pub fn start_fixed_update(&mut self) {
        self.fixed_time_accumulator += self.delta_real_seconds;
    }

    /// Checks to see if we should perform another fixed update iteration, and if so, returns true
    /// and reduces the accumulator.
    ///
    /// This should only be called by the engine.  Bad things might happen if you call this in
    /// your game.
    pub fn step_fixed_update(&mut self) -> bool {
        if self.fixed_time_accumulator >= self.fixed_seconds {
            self.fixed_time_accumulator -= self.fixed_seconds;
            true
        } else {
            false
        }
    }

    /// Updates the interpolation alpha factor given the current fixed update rate and accumulator.
    ///
    /// This should only be called by the engine.  Bad things might happen if you call this in
    /// your game.
    pub fn finish_fixed_update(&mut self) {
        self.interpolation_alpha = self.fixed_time_accumulator / self.fixed_seconds;
    }
}

impl Default for Time {
    fn default() -> Time {
        Time {
            delta_seconds: 0.0,
            delta_time: Duration::from_secs(0),
            delta_real_seconds: 0.0,
            delta_real_time: Duration::from_secs(0),
            fixed_seconds: duration_to_secs(Duration::new(0, 16_666_666)),
            fixed_time: Duration::new(0, 16_666_666),
            fixed_time_accumulator: 0.0,
            frame_number: 0,
            interpolation_alpha: 0.0,
            absolute_real_time: Duration::default(),
            absolute_time: Duration::default(),
            time_scale: 1.0,
        }
    }
}

/// A stopwatch which accurately measures elapsed time.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Stopwatch {
    /// Initial state with an elapsed time value of 0 seconds.
    Waiting,
    /// Stopwatch has started counting the elapsed time since this `Instant`
    /// and accumuluated time from previous start/stop cycles `Duration`.
    Started(Duration, Instant),
    /// Stopwatch has been stopped and reports the elapsed time `Duration`.
    Ended(Duration),
}

impl Default for Stopwatch {
    fn default() -> Stopwatch {
        Stopwatch::Waiting
    }
}

impl Stopwatch {
    /// Creates a new stopwatch.
    pub fn new() -> Stopwatch {
        Default::default()
    }

    /// Retrieves the elapsed time.
    pub fn elapsed(&self) -> Duration {
        match *self {
            Stopwatch::Waiting => Duration::new(0, 0),
            Stopwatch::Started(dur, start) => dur + start.elapsed(),
            Stopwatch::Ended(dur) => dur,
        }
    }

    /// Stops, resets, and starts the stopwatch again.
    pub fn restart(&mut self) {
        *self = Stopwatch::Started(Duration::new(0, 0), Instant::now());
    }

    /// Starts, or resumes, measuring elapsed time. If the stopwatch has been
    /// started and stopped before, the new results are compounded onto the
    /// existing elapsed time value.
    ///
    /// Note: Starting an already running stopwatch will do nothing.
    pub fn start(&mut self) {
        match *self {
            Stopwatch::Waiting => self.restart(),
            Stopwatch::Ended(dur) => {
                *self = Stopwatch::Started(dur, Instant::now());
            }
            _ => {}
        }
    }

    /// Stops measuring elapsed time.
    ///
    /// Note: Stopping a stopwatch that isn't running will do nothing.
    pub fn stop(&mut self) {
        if let Stopwatch::Started(dur, start) = *self {
            *self = Stopwatch::Ended(dur + start.elapsed());
        }
    }

    /// Clears the current elapsed time value.
    pub fn reset(&mut self) {
        *self = Stopwatch::Waiting;
    }
}

// Unit tests
#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use super::Stopwatch;

    // Timing varies more on macOS CI
    fn get_uncertainty() -> u32 {
        let is_macos = !std::env::var("MACOS").unwrap_or_default().is_empty();
        let is_ci = std::env::var("CI").is_ok();
        if is_macos && is_ci {
            20
        } else {
            10
        }
    }

    #[test]
    fn elapsed() {
        const DURATION: u64 = 1; // in seconds.
        let mut watch = Stopwatch::new();
        let uncertainty = get_uncertainty();

        watch.start();
        thread::sleep(Duration::from_secs(DURATION));
        watch.stop();

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::new(DURATION, 0);
        let lower = duration / 100 * (100 - uncertainty);
        let upper = duration / 100 * (100 + uncertainty);
        assert!(
            elapsed < upper && elapsed > lower,
            "expected {} +- {}% seconds, got {:?}",
            DURATION,
            uncertainty,
            elapsed
        );
    }

    #[test]
    fn reset() {
        const DURATION: u64 = 2; // in seconds.
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(DURATION));
        watch.stop();
        watch.reset();

        assert_eq!(0, watch.elapsed().subsec_nanos());
    }

    #[test]
    fn restart() {
        const DURATION0: u64 = 2; // in seconds.
        const DURATION: u64 = 1; // in seconds.
        let uncertainty = get_uncertainty(); // in percents.
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(DURATION0));
        watch.stop();

        watch.restart();
        thread::sleep(Duration::from_secs(DURATION));
        watch.stop();

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::new(DURATION, 0);
        let lower = duration / 100 * (100 - uncertainty);
        let upper = duration / 100 * (100 + uncertainty);
        assert!(
            elapsed < upper && elapsed > lower,
            "expected {} +- {}% seconds, got {:?}",
            DURATION,
            uncertainty,
            elapsed
        );
    }

    // test that multiple start-stop cycles are cumulative
    #[test]
    fn stop_start() {
        const DURATION: u64 = 3; // in seconds.
        let uncertainty = get_uncertainty(); // in percents.
        let mut watch = Stopwatch::new();

        for _ in 0..DURATION {
            watch.start();
            thread::sleep(Duration::from_secs(1));
            watch.stop();
        }

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::new(DURATION, 0);
        let lower = duration / 100 * (100 - uncertainty);
        let upper = duration / 100 * (100 + uncertainty);
        assert!(
            elapsed < upper && elapsed > lower,
            "expected {}  +- {}% seconds, got {:?}",
            DURATION,
            uncertainty,
            elapsed
        );
    }

    // Test that fixed_update methods accumulate and return correctly
    // Test confirms that with a fixed update of 120fps, we run fixed update twice with the timer
    // Runs at 10 times game speed, which shouldn't affect fixed updates
    #[test]
    fn fixed_update_120fps() {
        use super::Time;

        let mut time = Time::default();
        time.set_fixed_seconds(1.0 / 120.0);
        time.set_time_scale(10.0);

        let step = 1.0 / 60.0;
        let mut fixed_count = 0;
        for _ in 0..60 {
            time.set_delta_seconds(step);
            time.start_fixed_update();

            while time.step_fixed_update() {
                fixed_count += 1;
            }

            time.finish_fixed_update();
        }

        assert_eq!(fixed_count, 120);
    }

    // Test that fixed_update methods accumulate and return correctly
    // Test confirms that with a fixed update every 1 second, it runs every 1 second only
    #[test]
    fn fixed_update_1sec() {
        use super::Time;

        let mut time = Time::default();
        time.set_fixed_seconds(1.0);

        let step = 1.0 / 60.0;
        let mut fixed_count = 0;
        for _ in 0..130 {
            // Run two seconds
            time.set_delta_seconds(step);
            time.start_fixed_update();

            while time.step_fixed_update() {
                fixed_count += 1;
            }

            time.finish_fixed_update();
        }
        assert_eq!(fixed_count, 2);
    }
}

/// Converts a Duration to the time in seconds.
pub fn duration_to_secs(duration: Duration) -> f32 {
    duration.as_secs() as f32 + (duration.subsec_nanos() as f32 / 1.0e9)
}

/// Converts a Duration to the time in seconds in an f64.
pub fn duration_to_secs_f64(duration: Duration) -> f64 {
    duration.as_secs() as f64 + (f64::from(duration.subsec_nanos()) / 1.0e9)
}

/// Converts a time in seconds to a duration
pub fn secs_to_duration(secs: f32) -> Duration {
    Duration::new(secs as u64, ((secs % 1.0) * 1.0e9) as u32)
}

/// Converts a Duration to nanoseconds
pub fn duration_to_nanos(duration: Duration) -> u64 {
    (duration.as_secs() * 1_000_000_000) + u64::from(duration.subsec_nanos())
}

/// Converts nanoseconds to a Duration
pub fn nanos_to_duration(nanos: u64) -> Duration {
    Duration::new(nanos / 1_000_000_000, (nanos % 1_000_000_000) as u32)
}
