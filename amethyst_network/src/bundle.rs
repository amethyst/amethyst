use std::net::SocketAddr;

use serde::{de::DeserializeOwned, Serialize};

use amethyst_core::{
    bundle::{Result, ResultExt, SystemBundle},
    shred::DispatcherBuilder,
};

use crate::{filter::NetFilter, server::ServerConfig};

use super::NetSocketSystem;

/// A convenience bundle to create the infrastructure needed to send and receive network messages.
pub struct NetworkBundle<T> {
    /// the configuration used for the networking crate.
    config: ServerConfig,

    /// The filters applied on received network events.
    filters: Vec<Box<dyn NetFilter<T>>>,
}

impl<T> NetworkBundle<T> {
    /// Creates a new NetworkBundle.
    ///
    /// `receive_addr`: this is the address on which we will receive incoming packets.
    /// `send_addr`: this is the address from which we will send outgoing packets.
    pub fn new(
        receive_addr: SocketAddr,
        send_addr: SocketAddr,
        filters: Vec<Box<dyn NetFilter<T>>>,
    ) -> Self {
        let config = ServerConfig {
            udp_recv_addr: receive_addr,
            udp_send_addr: send_addr,
            max_throughput: 5000,
        };

        NetworkBundle { config, filters }
    }
}

impl<'a, 'b, T> SystemBundle<'a, 'b> for NetworkBundle<T>
where
    T: Send + Sync + PartialEq + Serialize + Clone + DeserializeOwned + 'static,
{
    /// Build the networking bundle by adding the networking system to the application.
    fn build(self, builder: &mut DispatcherBuilder<'_, '_>) -> Result<()> {
        let socket_system = NetSocketSystem::<T>::new(self.config, self.filters)
            .chain_err(|| "Failed to open network system.")?;

        builder.add(socket_system, "net_socket", &[]);

        Ok(())
    }
}
