use std::time::{Duration, Instant};
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
        15
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
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
    #[cfg(not(target_os = "macos"))]
    fn reset() {
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_millis(30));
        watch.stop();
        watch.reset();

        assert_eq!(0, watch.elapsed().subsec_nanos());
    }

    #[test]
    #[cfg(not(target_os = "macos"))]
    fn restart() {
        const DURATION0: u64 = 1000; // in milliseconds.
        const DURATION: u64 = 500; // in milliseconds.
        let uncertainty = get_uncertainty(); // in percents.
        let mut watch = Stopwatch::new();

        watch.start();
        thread::sleep(Duration::from_millis(DURATION0));
        watch.stop();

        watch.restart();
        thread::sleep(Duration::from_millis(DURATION));
        watch.stop();

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::from_millis(DURATION);
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
    #[cfg(not(target_os = "macos"))]
    fn stop_start() {
        let uncertainty = get_uncertainty(); // in percents.
        let mut watch = Stopwatch::new();

        for _ in 0..3 {
            watch.start();
            thread::sleep(Duration::from_millis(200));
            watch.stop();
        }

        // check that elapsed time was DURATION sec +/- UNCERTAINTY%
        let elapsed = watch.elapsed();
        let duration = Duration::from_millis(600);
        let lower = duration / 100 * (100 - uncertainty);
        let upper = duration / 100 * (100 + uncertainty);
        assert!(
            elapsed < upper && elapsed > lower,
            "expected {} +-{}% milliseconds, got {:?}",
            600,
            uncertainty,
            elapsed
        );
    }
}
