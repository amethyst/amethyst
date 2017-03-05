//! Utilities for working with time.

use std::time::{Duration, Instant};

/// A stopwatch which accurately measures elapsed time.
#[derive(PartialEq, Eq)]
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
        Stopwatch::Waiting
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
        match self {
            &mut Stopwatch::Waiting => {
                self.restart();
            }
            &mut Stopwatch::Ended(dur) => {
                *self = Stopwatch::Started(dur, Instant::now());
            }
            _ => {}
        }
    }

    /// Stops measuring elapsed time.
    ///
    /// Note: Stopping a stopwatch that isn't running will do nothing.
    pub fn stop(&mut self) {
        if let &mut Stopwatch::Started(dur, start) = self {
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
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_secs(2));
        watch.stop();

        // check that elapsed time was 2 sec +/- 1%
        let elapsed = watch.elapsed();
        let two_sec = Duration::new(2, 0);
        let lower = two_sec / 100 * 99;
        let upper = two_sec / 100 * 101;
        assert!(elapsed < upper && elapsed > lower,
                "expected about 2 seconds, got {:?}",
                elapsed);
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

        // check that elapsed time was 1 sec +/- 1%
        let elapsed = watch.elapsed();
        let one_sec = Duration::new(1, 0);
        let lower = one_sec / 100 * 99;
        let upper = one_sec / 100 * 101;
        assert!(elapsed < upper && elapsed > lower,
                "expected about 1 second, got {:?}",
                elapsed);
    }

    // test that multiple start-stop cycles are cumulative
    #[test]
    fn stop_start() {
        let mut watch = Stopwatch::new();

        for _ in 0..3 {
            watch.start();
            thread::sleep(Duration::from_secs(1));
            watch.stop();
        }

        // check that elapsed time was 3 sec +/- 1%
        let elapsed = watch.elapsed();
        let three_sec = Duration::new(3, 0);
        let lower = three_sec / 100 * 99;
        let upper = three_sec / 100 * 101;
        assert!(elapsed < upper && elapsed > lower,
                "expected about 3 seconds, got {:?}",
                elapsed);
    }
}
