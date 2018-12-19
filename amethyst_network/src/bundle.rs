use std::net::SocketAddr;

use serde::{de::DeserializeOwned, Serialize};

use amethyst_core::{
    bundle::{Result, ResultExt, SystemBundle},
    shred::DispatcherBuilder,
};

use crate::filter::NetFilter;
use crate::metrics::{MetricObserver, NetworkMetricObject, NetworkMetrics};

use super::NetSocketSystem;

/// A convenience bundle to create the infrastructure needed to send and receive network messages.
pub struct NetworkBundle<T> {
    /// Local socket address
    addr: SocketAddr,

    /// The filters applied on received network events.
    filters: Vec<Box<dyn NetFilter<T>>>,

    /// These list of metrics observers, who are capable of observing metrics changes.
    metrics: Vec<Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>>,
}

impl<T> NetworkBundle<T> {
    /// Creates a new NetworkBundle that connects to the `addr`.
    pub fn new(addr: SocketAddr, filters: Vec<Box<dyn NetFilter<T>>>) -> Self {
        NetworkBundle {
            addr,
            filters,
            metrics: Vec::new(),
        }
    }

    pub fn with_metric(
        mut self,
        metric: Box<dyn MetricObserver<NetworkMetricObject> + Send + Sync>,
    ) -> NetworkBundle<T> {
        self.metrics.push(metric);
        self
    }
}

impl<'a, 'b, T> SystemBundle<'a, 'b> for NetworkBundle<T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder<'_, '_>) -> Result<()> {
        let mut socket_system = NetSocketSystem::<T>::new(self.addr, self.filters)
            .chain_err(|| "Failed to open network system.")?;

        socket_system.set_metrics(self.metrics);

        builder.add(socket_system, "net_socket", &[]);

        Ok(())
    }
}
