//! Utilities for working with time.

use std::time::{Duration, Instant};

/// Frame timing values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Time {
    /// Time elapsed since the last frame.
    pub delta_time: Duration,
    /// Rate at which `State::fixed_update` is called.
    pub fixed_step: Duration,
    /// Time at which `State::fixed_update` was last called.
    pub last_fixed_update: Instant,
}

impl Default for Time {
    fn default() -> Time {
        Time {
            delta_time: Duration::from_secs(0),
            fixed_step: Duration::new(0, 16666666),
            last_fixed_update: Instant::now(),
        }
    }
}

/// A stopwatch which accurately measures elapsed time.
#[derive(Debug, Eq, PartialEq)]
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
        Stopwatch::default()
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
    use super::Stopwatch;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn elapsed() {
        const DURATION: u64 = 1; // in seconds.
        const UNCERTAINTY: u32 = 5; // in percents.
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(DURATION));
        watch.stop();

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::new(DURATION, 0);
        let lower = duration / 100 * (100 - UNCERTAINTY);
        let upper = duration / 100 * (100 + UNCERTAINTY);
        assert!(elapsed < upper && elapsed > lower,
                "expected {} +- {}% seconds, got {:?}",
                DURATION,
                UNCERTAINTY,
                elapsed);
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
        const UNCERTAINTY: u32 = 5; // in percents.
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
        let lower = duration / 100 * (100 - UNCERTAINTY);
        let upper = duration / 100 * (100 + UNCERTAINTY);
        assert!(elapsed < upper && elapsed > lower,
                "expected {} +- {}% seconds, got {:?}",
                DURATION,
                UNCERTAINTY,
                elapsed);
    }

    // test that multiple start-stop cycles are cumulative
    #[test]
    fn stop_start() {
        const DURATION: u64 = 3; // in seconds.
        const UNCERTAINTY: u32 = 5; // in percents.
        let mut watch = Stopwatch::new();

        for _ in 0..DURATION {
            watch.start();
            thread::sleep(Duration::from_secs(1));
            watch.stop();
        }

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::new(DURATION, 0);
        let lower = duration / 100 * (100 - UNCERTAINTY);
        let upper = duration / 100 * (100 + UNCERTAINTY);
        assert!(elapsed < upper && elapsed > lower,
                "expected {}  +- {}% seconds, got {:?}",
                DURATION,
                UNCERTAINTY,
                elapsed);
    }
}
