use super::{MetricObserver, NetworkMetricObject};
use std::time::Instant;

/// This type is responsible for handling the network metrics.
///
/// _This struct is the 'subject' of the observer pattern and is responsible for notifying all observers_
pub struct NetworkMetrics {
    // observers who are watching for changes in metrics.
    observers: Vec<Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>>,
    // the object containing the metrics.
    metric: NetworkMetricObject,
}

impl NetworkMetrics {
    /// Create a new instance of `NetworkMetrics` by passing in the observers who will be watching metrics changes.
    pub fn new(
        metrics: Vec<Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>>,
    ) -> NetworkMetrics {
        NetworkMetrics {
            observers: metrics,
            metric: Default::default(),
        }
    }

    /// Register an incoming packet and notify all metrics observers.
    pub fn add_incoming_packet(&mut self, count: u16) {
        self.metric.total_incoming_packets += count as u32;
        self.metric.throughput_counter += count as u32;

        if self.metric.throughput_monitor.elapsed() >= self.metric.throughput_elapse {
            self.metric.throughput = self.metric.throughput_counter;
            self.metric.throughput_monitor = Instant::now();
            self.metric.throughput_counter = 0;
        }

        self.notify_observers();
    }

    /// Register an outgoing packet and notify all metrics observers.
    pub fn add_outgoing_packets(&mut self, count: u16) {
        self.metric.total_outgoing_packets += count as u32;
        self.notify_observers();
    }

    /// Notify all metrics observers.
    fn notify_observers(&mut self) {
        for observer in self.observers.iter_mut() {
            observer.update(&mut self.metric);
        }
    }
}
