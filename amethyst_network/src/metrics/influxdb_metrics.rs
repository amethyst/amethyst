use super::{MetricObserver, NetworkMetricObject};
use serde::Serialize;
use std::time::{Duration, Instant};

/// This will handle changing metrics and write them to InfluxDB.
/// InfluxDB write interval could be managed by passing in a `Duration` at creation time.
pub struct InfluxDBMetrics {
    // this will be used for the writing interval to the console.
    last_send: Instant,
    // interval rate
    update_rate: Duration,
}

impl InfluxDBMetrics {
    /// Create a new instance of `InfluxDBMetrics` by passing in a interval and the influxdb connection information which the metrics will be written to the passed csv-file.
    pub fn new(
        update_rate: Duration,
        host_name: &str,
        username: &str,
        password: &str,
    ) -> InfluxDBMetrics {
        // todo: make influx connection with extern crate.
        unimplemented!()
    }
}

impl MetricObserver<NetworkMetricObject> for InfluxDBMetrics {
    fn update(&mut self, metrics_object: &NetworkMetricObject) {
        if self.last_send.elapsed() >= self.update_rate {
            self.write(metrics_object);
        }
    }

    fn write(&self, metrics_object: &NetworkMetricObject) {
        // todo:
        // 1) set measure
        // 2) set fields
        // 3) write to influx
    }
}
