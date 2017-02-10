//! Utilities for working with time.

use std::time::{Duration, Instant};

/// A stopwatch which accurately measures elapsed time.
#[derive(PartialEq, Eq)]
pub enum Stopwatch {
    /// Initial state with an elapsed time value of 0 seconds.
    Waiting,
    /// Stopwatch has started counting the elapsed time since this `Instant`.
    Started(Instant),
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
        Stopwatch::Waiting
    }

    /// Retrieves the elapsed time.
    pub fn elapsed(&self) -> Duration {
        match self {
            &Stopwatch::Waiting => Duration::new(0, 0),
            &Stopwatch::Started(start) => start.elapsed(),
            &Stopwatch::Ended(dur) => dur,
        }
    }

    /// Stops, resets, and starts the stopwatch again.
    pub fn restart(&mut self) {
        *self = Stopwatch::Started(Instant::now());
    }

    /// Starts, or resumes, measuring elapsed time. If the stopwatch has been
    /// started and stopped before, the new results are compounded onto the
    /// existing elapsed time value.
    ///
    /// Note: Starting an already running stopwatch will do nothing.
    pub fn start(&mut self) {
        if self == &Stopwatch::Waiting {
            self.restart();
        }
    }

    /// Stops measuring elapsed time.
    ///
    /// Note: Stopping a stopwatch that isn't running will do nothing.
    pub fn stop(&mut self) {
        if let &mut Stopwatch::Started(start) = self {
            *self = Stopwatch::Ended(start.elapsed());
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
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(2));
        watch.stop();

        assert_eq!(2, watch.elapsed().as_secs());
    }

    #[test]
    fn reset() {
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(2));
        watch.stop();
        watch.reset();

        assert_eq!(0, watch.elapsed().subsec_nanos());
    }

    #[test]
    fn restart() {
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(2));
        watch.stop();

        watch.restart();
        thread::sleep(Duration::from_secs(1));
        watch.stop();

        assert_eq!(1, watch.elapsed().as_secs());
    }
}
