use super::{MetricObserver, NetworkMetricObject};
use serde::Serialize;
use std::fs::File;
use std::time::{Duration, Instant};

struct Timeout {

}

/// This will handle changing metrics and write them to the console.
/// Console write interval could be managed by passing in a `Duration` at creation time.
pub struct ConsoleMetrics {
    // this will be used for the writing interval to the console.
    last_send: Instant,
    // interval rate
    update_rate: Duration,
}

impl ConsoleMetrics {
    /// Create a new instance of `ConsoleMetrics` by passing in a interval at which the metrics will be written to screen.
    pub fn new(update_rate: Duration) -> ConsoleMetrics {
        ConsoleMetrics {
            last_send: Instant::now(),
            update_rate,
        }
    }
}

impl MetricObserver<NetworkMetricObject> for ConsoleMetrics {
    fn update(&mut self, metrics_object: &NetworkMetricObject) {
        if self.last_send.elapsed() >= self.update_rate {
            self.write(metrics_object);
        }
        self.last_send = Instant::now();
    }

    fn write(&self, metrics_object: &NetworkMetricObject) {
        println!("{:?}", metrics_object);
    }
}
