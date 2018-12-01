use std::net::SocketAddr;

use serde::{de::DeserializeOwned, Serialize};

use amethyst_core::{
    bundle::{Result, ResultExt, SystemBundle},
    SimpleDispatcherBuilder,
};

use crate::filter::NetFilter;

use super::NetSocketSystem;

/// A convenience bundle to create the infrastructure needed to send and receive network messages.
pub struct NetworkBundle<T> {
    /// Local socket address
    addr: SocketAddr,

    /// The filters applied on received network events.
    filters: Vec<Box<dyn NetFilter<T>>>,
}

impl<T> NetworkBundle<T> {
    /// Creates a new NetworkBundle that connects to the `addr`.
    pub fn new(addr: SocketAddr, filters: Vec<Box<dyn NetFilter<T>>>) -> Self {
        NetworkBundle { addr, filters }
    }
}

impl<'a, 'b, 'c, T, D> SystemBundle<'a, 'b, 'c, D> for NetworkBundle<T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
    D: SimpleDispatcherBuilder<'a, 'b, 'c>,
{
    fn build(self, builder: &mut D) -> Result<()> {
        let socket_system = NetSocketSystem::<T>::new(self.addr, self.filters)
            .chain_err(|| "Failed to open network system.")?;

        builder.add(socket_system, "net_socket", &[]);

        Ok(())
    }
}
