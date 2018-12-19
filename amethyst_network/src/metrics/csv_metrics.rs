use super::{MetricObserver, NetworkMetricObject};
use serde::Serialize;
use std::fs::File;
use std::time::{Duration, Instant};

/// This will handle changing metrics and write them to a CSV file.
/// CSV write interval could be managed by passing in a `Duration` at creation time.
pub struct CsvMetrics {
    // the file handle to the CSV file
    csv_file: File,
    // this will be used for the writing interval to the console.
    last_send: Instant,
    // interval rate
    update_rate: Duration,
}

impl CsvMetrics {
    /// Create a new instance of `CsvMetrics` by passing in a interval at which the metrics will be written to the passed csv-file.
    pub fn new(csv_file: File, update_rate: Duration) -> CsvMetrics {
        CsvMetrics {
            csv_file,
            last_send: Instant::now(),
            update_rate,
        }
    }
}

impl MetricObserver<NetworkMetricObject> for CsvMetrics {
    fn update(&mut self, metrics_object: &NetworkMetricObject) {
        if self.last_send.elapsed() >= self.update_rate {
            self.write(metrics_object);
        }
    }

    fn write(&self, metrics_object: &NetworkMetricObject) {
        // todo:
        // 1) serialize object to csv
    }
}
