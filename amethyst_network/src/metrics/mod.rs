//! Module for managing network metrics.

use serde::Serialize;
use std::time::{Duration, Instant};
use std::thread;


mod console_metrics;
mod network_metrics;

pub use self::console_metrics::ConsoleMetrics;
pub use self::network_metrics::NetworkMetrics;

/// Metrics observer which could be implemented to observe metrics changes.
/// This trait could be implemented for handling metrics a certain way.
pub trait MetricObserver<T: Sync + Send> {
    /// This method will be called to when a change in some metric has occurred.
    fn update(&mut self, metrics_object: &T);
    /// This method could be used to write the object to the implemented metric output.
    fn write(&self, metrics_object: &T);
}

/// Object containing network metrics.
#[derive(Debug, Copy, Clone)]
pub struct NetworkMetricObject {
    /// The throughput in seconds
    pub throughput: u32,
    /// The throughput counter to count packets until 'throughput_monitor' as elapsed 1 second
    /// This will be reset to 0 after one second elapsed
    pub throughput_counter: u32,
    /// The total incoming packets
    pub total_incoming_packets: u32,
    /// The total outgoing packets
    pub total_outgoing_packets: u32,
    /// The counter which will be used to monitor incoming packets during 1 second
    pub throughput_monitor: Instant,
    /// The elapse time for monitoring throughput
    throughput_elapse: Duration,
}

impl Default for NetworkMetricObject {
    fn default() -> Self {
        NetworkMetricObject {
            throughput: 0,
            throughput_counter: 0,
            total_incoming_packets: 0,
            total_outgoing_packets: 0,
            throughput_monitor: Instant::now(),
            throughput_elapse: Duration::from_millis(1000),
        }
    }
}

impl From<ConsoleMetrics> for Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync> {
    fn from(console_metrics: ConsoleMetrics) -> Self {
        Box::new(console_metrics) as Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>
    }
}
