use super::NetSocketSystem;
use amethyst_core::bundle::{Result, SystemBundle};
use amethyst_core::shred::DispatcherBuilder;
use filter::NetFilter;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::net::SocketAddr;

/// A convenience bundle to create the infrastructure needed to send and receive network messages.
pub struct NetworkBundle<T> {
    /// Local socket address
    addr: SocketAddr,

    /// The filters applied on received network events.
    filters: Vec<Box<NetFilter<T>>>,
}

impl<T> NetworkBundle<T> {
    /// Creates a new NetworkBundle that connects to the `addr`.
    pub fn new(addr: SocketAddr, filters: Vec<Box<NetFilter<T>>>) -> Self {
        NetworkBundle { addr, filters }
    }
}

impl<'a, 'b, T> SystemBundle<'a, 'b> for NetworkBundle<T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
{
    fn build(self, builder: &mut DispatcherBuilder) -> Result<()> {
        let socket_system = NetSocketSystem::<T>::new(self.addr, self.filters)
            .expect("Failed to open network system.");

        builder.add(socket_system, "net_socket", &[]);

        Ok(())
    }
}
