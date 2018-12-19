use super::{MetricObserver, NetworkMetricObject};
use std::time::Instant;
use std::thread::{self, JoinHandle};
use std::sync::mpsc::{self, Sender, SyncSender, Receiver};

/// This type is responsible for handling the network metrics.
///
/// _This struct is the 'subject' of the observer pattern and is responsible for notifying all observers_
pub struct NetworkMetrics {
    // the object containing the metrics.
    metric: NetworkMetricObject,

    metrics_scheduler: MetricsScheduler
}

impl NetworkMetrics {
    /// Create a new instance of `NetworkMetrics` by passing in the observers who will be watching metrics changes.
    pub fn new(
        metrics: Vec<Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>>,
    ) -> NetworkMetrics {
        NetworkMetrics {
            metric: Default::default(),
            metrics_scheduler: MetricsScheduler::new(metrics)
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

        self.schedule_metrics_write();
    }

    /// Register an outgoing packet and notify all metrics observers.
    pub fn add_outgoing_packets(&mut self, count: u16) {
        self.metric.total_outgoing_packets += count as u32;
        self.schedule_metrics_write();
    }

    fn schedule_metrics_write(&mut self) {
        self.metrics_scheduler.schedule(&self.metric);
    }
}


struct MetricsScheduler {
    metrics_sender: SyncSender<NetworkMetricObject>,
    send_buffer_counter: u16,
    handle: JoinHandle<()>
}

impl MetricsScheduler {
    pub fn new(metrics: Vec<Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>>) -> MetricsScheduler {
        let (tx, rx) = mpsc::sync_channel(1000);
        let mut metrics = metrics;

        let handle = thread::spawn(move || {
            loop {
                if let Ok(ref mut value) = rx.recv() {
                    for observer in metrics.iter_mut() {
                        observer.update(value);
                    }
                }
            }
        });

        MetricsScheduler {
            metrics_sender: tx,
            send_buffer_counter: 0,
            handle
        }
    }

    pub fn schedule(&mut self, object: &NetworkMetricObject) {
        self.metrics_sender.try_send(*object);
    }
}
