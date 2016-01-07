//! Utilities for working with time.

extern crate time;

pub use self::time::{Duration, SteadyTime};

/// Useful utility for accurately measuring elapsed time.
pub struct Stopwatch {
    start_time: SteadyTime,
    end_time: SteadyTime,
    running: bool,
}

impl Stopwatch {
    pub fn new() -> Stopwatch {
        let initial_time = SteadyTime::now();

        Stopwatch {
            start_time: initial_time,
            end_time: initial_time,
            running: false,
        }
    }

    /// Retrieves the elapsed time.
    pub fn elapsed(&self) -> Duration {
        self.end_time - self.start_time
    }

    /// Stops, resets, and starts the stopwatch again.
    pub fn restart(&mut self) {
        self.reset();
        self.start();
    }

    /// Starts, or resumes, measuring elapsed time. If the stopwatch has been
    /// started and stopped before, the new results are compounded onto the
    /// existing elapsed time value.
    ///
    /// Note: Starting an already running stopwatch will do nothing.
    pub fn start(&mut self) {
        if !self.running {
            if self.elapsed() == Duration::seconds(0) {
                self.reset()
            }

            self.running = true;
        }
    }

    /// Stops measuring elapsed time.
    ///
    /// Note: Stopping a stopwatch that isn't running will do nothing.
    pub fn stop(&mut self) {
        if self.running {
            self.end_time = SteadyTime::now();
            self.running = false;
        }
    }

    /// Clears the current elapsed time value.
    pub fn reset(&mut self) {
        self.start_time = SteadyTime::now();
        self.end_time = self.start_time;
    }
}

// Unit tests
#[cfg(test)]
mod test {
    use super::*;
    use std::thread;
    use std::time;

    #[test]
    fn elapsed() {
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(time::Duration::from_secs(2));
        watch.stop();

        assert_eq!(2, watch.elapsed().num_seconds());
    }

    #[test]
    fn reset() {
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(time::Duration::from_secs(2));
        watch.stop();
        watch.reset();

        assert_eq!(0, watch.elapsed().num_nanoseconds().unwrap());
    }

    #[test]
    fn restart() {
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(time::Duration::from_secs(2));
        watch.stop();

        watch.restart();
        thread::sleep(time::Duration::from_secs(1));
        watch.stop();

        assert_eq!(1, watch.elapsed().num_seconds());
    }
}
